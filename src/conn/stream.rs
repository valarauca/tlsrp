use super::fault::Fault;
use super::super::mio::{
    Poll,
    Ready,
    Event,
    Token, 
    PollOpt
};
use super::super::mio::tcp::{
    TcpStream,
    Shutdown
};
use super::super::mio::deprecated::{
    UnixStream, 
    Shutdown as UnixShutdown
};
use super::super::native_tls::{
    TlsStream,
    MidHandshakeTlsStream, 
    Error as TLSError, 
    HandshakeError,
    TlsAcceptor
};
use std::io;
use std::io::prelude::*;


///Determines the type of stream
pub trait StreamType {
    fn is_uninitialized(&self) -> bool;
    fn is_handshaking(&self) -> bool;
    fn is_tcp(&self) -> bool;
    fn is_tls(&self) -> bool;
    fn is_unix(&self) -> bool;
}


///Encapsulates all types of data in a streams which can be handled by MIO
pub enum Stream {
    Tls(TlsStream<TcpStream>),
    Tcp(TcpStream),
    Unix(UnixStream),
    Uninitialized,
    TlsHandShake(MidHandshakeTlsStream<TcpStream>)
}
#[test]
fn test_stream_size() {
    use std::mem::size_of;
    assert_eq!( size_of::<Stream>(), 56);
}
impl Stream {
    
  
    ///Consumes self and attempts to continue a handshaking process. For all other stream
    ///types it will return an `Ok(Self)` of the orginal stream type without modifying it
    ///Generally this method is only ever meant to be called on the TlsHandShake type
    ///and that should be checked before hand.
    #[inline(always)]
    pub fn handshake(self) -> Result<Stream,Fault> {
        match self {
            Stream::TlsHandShake(x) => match x.handshake() {
                Ok(x) => Ok(Stream::Tls(x)),
                Err(HandshakeError::Failure(e)) => Err(Fault::from(e)),
                Err(HandshakeError::Interrupted(x)) => Ok(Stream::TlsHandShake(x))
            },
            Stream::Uninitialized => Ok(Stream::Uninitialized),
            Stream::Tls(x) => Ok(Stream::Tls(x)),
            Stream::Tcp(x) => Ok(Stream::Tcp(x)),
            Stream::Unix(x) => Ok(Stream::Unix(x))
        }
    }

    ///Build a new TCP Stream. Also register it with the Epoll Interface
    pub fn create_tcp(x: TcpStream, poll: &Poll, t: Token) -> Result<Stream,Fault> {
        let r = Ready::readable();
        match poll.register(&x, t, r, PollOpt::level()) {
            Ok(_) => Ok(Stream::Tcp(x)),
            Err(e) => {
                let _ = x.shutdown(Shutdown::Both);
                Err(Fault::from(e))
            }
        }
    }
    ///Build a new UnixStream. ALso register it with Epoll Interface
    pub fn create_unix(x: UnixStream, poll: &Poll, t: Token) -> Result<Stream,Fault> {
        let r = Ready::readable();
        match poll.register(&x, t, r, PollOpt::level()) {
            Ok(_) => Ok(Stream::Unix(x)),
            Err(e) => {
                let _ = x.shutdown(UnixShutdown::Both);
                Err(Fault::from(e))
            }
        }
    }
    ///Build a new TLS Stream. Well start the handshaking. The code flow
    ///exists for this to _potentially_ finish. But this is likely impossible
    ///as a `WOULDBLOCK` error _should_ occur before that.
    pub fn create_tls(x: TcpStream, poll: &Poll, t: Token, a: &TlsAcceptor) -> Result<Stream,Fault> {
        let r = Ready::readable();
        match poll.register(&x, t, r, PollOpt::level()) {
            Ok(_) => { },
            Err(e) => {
                let _ = x.shutdown(Shutdown::Both);
                return Err(Fault::from(e));
            }
        };
        match a.accept(x) {
            Ok(x) => Ok(Stream::Tls(x)),
            Err(HandshakeError::Failure(e)) => Err(Fault::from(e)),
            Err(HandshakeError::Interrupted(x)) => Ok(Stream::TlsHandShake(x))
        }
    }
}
impl StreamType for Stream {
    #[inline(always)]
    fn is_uninitialized(&self) -> bool {
        match self {
            &Stream::Uninitialized => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_handshaking(&self) -> bool {
        match self {
            &Stream::TlsHandShake(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_unix(&self) -> bool {
        match self {
            &Stream::Unix(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_tcp(&self) -> bool {
        match self {
            &Stream::Tcp(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    fn is_tls(&self) -> bool {
        match self {
            &Stream::Tls(_) => true,
            _ => false
        }
    }
}
impl Read for Stream {

    ///Uninitialized streams return a usize::MAX value. As returning 0 may cause
    ///some systems to think and EOF has occured. 
    ///At the same time reading/writing from a TLS HandShake will cause them to
    ///by-pass the encrypt layer. Which is ya know... bad. So don't do that.
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            &mut Stream::Uninitialized => Ok(::std::usize::MAX),
            &mut Stream::TlsHandShake(ref mut x) => x.get_mut().read(buf),
            &mut Stream::Unix(ref mut x) => x.read(buf),
            &mut Stream::Tcp(ref mut x) => x.read(buf),
            &mut Stream::Tls(ref mut x) => x.read(buf)
        }
    }
}
impl Write for Stream {
    ///Uninitialized streams return a usize::MAX value. As returning 0 may cause
    ///some systems to think and EOF has occured. 
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            &mut Stream::Uninitialized => Ok(::std::usize::MAX),
            &mut Stream::TlsHandShake(ref mut x) => x.get_mut().write(buf),
            &mut Stream::Unix(ref mut x) => x.write(buf),
            &mut Stream::Tcp(ref mut x) => x.write(buf),
            &mut Stream::Tls(ref mut x) => x.write(buf)
        }
    }
    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        match self {
            &mut Stream::Uninitialized => Ok(()),
            &mut Stream::TlsHandShake(ref mut x) => x.get_mut().flush(),
            &mut Stream::Unix(ref mut x) => x.flush(),
            &mut Stream::Tcp(ref mut x) => x.flush(),
            &mut Stream::Tls(ref mut x) => x.flush()
        }
    }
}

#[test]
fn test_stream_type() {
    use std::mem::{uninitialized,forget};
    
    let ui = Stream::Uninitialized;
    let hs = unsafe{ Stream::TlsHandShake(uninitialized())};
    let un = unsafe{ Stream::Unix(uninitialized()) };
    let tc = unsafe{ Stream::Tcp(uninitialized()) };
    let tl = unsafe{ Stream::Tls(uninitialized()) };

    assert!( ui.is_uninitialized() );
    assert!( hs.is_handshaking() );
    assert!( un.is_unix() );
    assert!( tc.is_tcp() );
    assert!( tl.is_tls() );
    
    
    forget(ui);
    forget(hs);
    forget(un);
    forget(tc);
    forget(tl);
}
