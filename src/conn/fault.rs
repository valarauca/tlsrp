
use super::super::rustls::TLSError;
use std::io::Error as OSFault;

pub enum Fault {
	TLS(TLSError),
	OS(OSFault)
}
impl From<OSFault> for Fault {
	fn from(x: OSFault) -> Fault {
		Fault::OS(x)
	}
}
impl From<TLSError> for Fault {
	fn from(x: TLSError) -> Fault {
		Fault::TLS(x)
	}
}
