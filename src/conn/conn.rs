
use std::io::prelude::*;
use super::fault::Fault;
use super::status::Status;
use super::super::mio::tcp::TcpStream;
use super::super::mio::{Token,Poll, Ready, PollOpt};
use super::super::rustls::{ServerSession,ServerConfig,Session};
use std::sync::atomic::{AtomicUsize,Ordering};
use std::io;
use std::net::SocketAddr;

///Describes a connection
pub struct Connection {
    pub addr: SocketAddr,
    stream: TcpStream,
    pub token: Token,
    tls: ServerSession,
    pub mode: Status,
    pub incoming: Option<Ready>,
}
impl Connection {
    
    ///Get a new connection
    pub fn new(addr: SocketAddr, stream: TcpStream, token: Token, tls: ServerSession) -> Connection {
        Connection {
            addr: addr,
            stream: stream,
            token: token,
            tls: tls,
            mode: Status::HandShake
        }
    }

    ///Detects the rising/falling edge of handshaking
    pub fn update_handshaking(&mut self) {
        let st = self.mode.is_handshake();
        let tl = self.tls.is_handshaking();
        if st& !tl {
            self.mode = Status::Http;
        }
        if tl && self.mode.is_http() {
            self.mode = Status::HandShake;
        }
    }

    ///send client close
    pub fn close_client(&mut self) {
        self.tls.send_close_notify()
    }

    ///wants to close
    pub fn closing(&self) -> bool {
        self.mode.is_closing()
    }

    ///register
    pub fn register(&mut self, poll: &Poll, oneshot: bool) {
        if self.token == Token(0) {
            return;
        }
        let opt = if oneshot { PollOpt::oneshot() | PollOpt::level() } else { PollOpt::level() };
        let ready = Ready::readable()|Ready::writable();
        match poll.register( &self.stream, self.token, ready, opt) {
            Ok(_) => { },
            Err(e) => self.mode = Status::from(e)
        };
    }

    ///change token
    pub fn new_token(&mut self, t: Token) {
        self.token = t;
    }

    ///write TLS
    pub fn tls_write(&mut self) {
        let flag = self.tls.write_tls(&mut self.stream);
        if let Some(err) = flag.err() {
            self.mode = Status::from(err);
        }
    }

    ///Attempt to read
    pub fn tls_read(&mut self) {
        match self.tls.read_tls(&mut self.stream) {
            Ok(0) => {
                self.mode = Status::Closing(None);
                return;
            },
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    self.mode = Status::from(e);
                }
                return;
            },
            _ => { }
        };
        let flag_ = self.tls.process_new_packets();
        if let Some(err) = flag_.err() {
            self.mode = Status::from(err);
        }
    }

    ///Read clear text data
    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) {
        match self.tls.read_to_end(buf) {
            Err(e) => if e.kind() != io::ErrorKind::WouldBlock {
                    self.mode = Status::from(e);
            },
            Ok(_) => { }
        };
    }
    
    ///Write clear text
    pub fn write_all(&mut self, buf: &[u8]) {
        match self.tls.write_all(buf) {
            Err(e) => self.mode = Status::from(e),
            Ok(_) => { }
        };
    }
}
