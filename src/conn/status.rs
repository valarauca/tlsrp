
use super::fault::Fault;
use std::io::Error as IO;
use super::super::rustls::TLSError;

///Describes the status of the connection is in
pub enum Status {
    HandShake,
    Closing(Option<Fault>),
    Http,
	Closed,
}
impl Status {
    #[inline(always)]
    pub fn is_handshake(&self) -> bool {
        match *self {
            Status::HandShake => true,
            _ => false
        }
    }
    #[inline(always)]
    pub fn is_closing(&self) -> bool {
        match *self {
            Status::Closing(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    pub fn is_http(&self) -> bool {
        match *self {
            Status::Http => true,
            _ => false
        }
    }
	#[inline(always)]
	pub fn is_closed(&self) -> bool {
		match *self {
			Status::Closed => true,
			_ => false
        }
	}
}
impl From<Fault> for Status {
    fn from(x: Fault) -> Self {
        Status::Closing(Some(x))
    }
}
impl From<IO> for Status {
    fn from(x: IO) -> Self {
        Status::Closing(Some(Fault::OS(x)))
    }
}
impl From<TLSError> for Status {
    fn from(x: TLSError) -> Self {
        Status::Closing(Some(Fault::TLS(x)))
    }
}
