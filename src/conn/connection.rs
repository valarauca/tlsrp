use std::mem::replace;
use std::io;
use std::io::prelude::*;
use super::super::mio::{
    Token,
    Ready,
};
use super::super::lock::{
    Lock,
    Locky
};
use super::stream::{
    Stream,
    StreamType
};
use super::fault::Fault;
use super::super::workerid::WorkerID;

///Represents a single connection. Should be the size of 3 cache lines.
#[repr(C)]
#[allow(dead_code)]
pub struct Connection {
    lock: Lock,
    //cache line
    pub token: Token,
    data: Stream,
    //cache line
    pub other: Token, 
    pub err: Fault,
    pub action: Ready,
    pad: u64
}
unsafe impl Sync for Connection { }
#[test]
fn test_connection_size() {
    use std::mem::size_of;

    assert_eq!( size_of::<Connection>(), 192);
}

impl Connection {

    ///Constructs a default connection. Nothing inside it is active
    pub fn new() -> Connection{
        Connection {
            lock: Lock::new(),
            token: Token(0),
            data: Stream::Uninitialized,
            other: Token(0),
            err: Fault::None,
            action: Ready::none(),
            pad: 0u64
        }
    }

    ///Set up a stream
    pub fn setup(&mut self, x: Stream, w: WorkerID) -> Result<(),Stream> {
        let x = replace( &mut self.data, x );
        self.lock.manual_set(w);
        if ! x.is_uninitialized() {
            Err(x)
        } else {
            Ok(())
        }
    
    }

    ///Attempt to replace a stream. Returns a Err(Stream) if a
    ///an actual initialized stream is replaced. This _shouldn't_
    ///ever happen, but yeah how the Stream sum type works it
    ///_can_ so we have to account for that eventuallity
    pub fn replace(&mut self, x: Stream) -> Result<(), Stream> {
        let x = replace( &mut self.data, x );
        if ! x.is_uninitialized() {
            Err(x)
        } else {
            Ok(())
        }
    }
   
    ///Attempt to handshake on a value. Returns a flag if the hand shaking
    ///is complete (or the value is already a tls stream, technically).
    pub fn handshake(&mut self) -> Result<bool,Fault> {
        let x = replace(&mut self.data, Stream::Uninitialized);
        match x.handshake() {
            Ok(x) => {
                let flag = x.is_tls();
                let _ = replace(&mut self.data, x);
                Ok(flag)
            },
            Err(e) => Err(e)
        }
    }
    ///Checks if the token is nonzero
    #[inline(always)]
    pub fn token_valid(&self) -> bool {
        self.token.0 != 0
    }

    ///check if the connection has a partner
    #[inline(always)]
    pub fn has_partner(&self) -> bool {
        self.other.0 != 0
    }

    ///is readable
    #[inline(always)]
    pub fn wants_read(&self) -> bool {
        self.action == Ready::readable()
    }
}
#[test]
fn test_connection() {
    let x = Connection::new();
    assert!( ! x.has_partner() );
    assert!( ! x.token_valid() );
    assert!( x.is_uninitialized() );
    assert_eq!( x.action, Ready::none() );
}
impl Locky for Connection {

    #[inline(always)]
    fn try_lock(&self) -> bool {
        self.lock.try_lock()
    }
    #[inline(always)]
    fn unlock(&self) {
        self.lock.unlock()
    }
    #[inline(always)]
    fn spinlock(&self) {
        self.lock.spinlock();
    }
    #[inline(always)]
    fn worker(&self) -> Option<WorkerID> {
        self.lock.worker()
    }
}
impl StreamType for Connection {
    #[inline(always)]
    fn is_uninitialized(&self) -> bool {
        match &self.data {
            &Stream::Uninitialized => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_handshaking(&self) -> bool {
        match &self.data {
            &Stream::TlsHandShake(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_unix(&self) -> bool {
        match &self.data {
            &Stream::Unix(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_tcp(&self) -> bool {
        match &self.data {
            &Stream::Tcp(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_tls(&self) -> bool {
        match &self.data {
            &Stream::Tls(_) => true,
            _ => false
        }
    }
}
impl Read for Connection {

    ///Uninitialized streams return a usize::MAX value. As returning 0 may cause
    ///some systems to think and EOF has occured. 
    ///At the same time reading/writing from a TLS HandShake will cause them to
    ///by-pass the encrypt layer. Which is ya know... bad. So don't do that.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}
impl Write for Connection {
    ///Uninitialized streams return a usize::MAX value. As returning 0 may cause
    ///some systems to think and EOF has occured. 
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}

