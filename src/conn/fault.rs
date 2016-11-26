
use super::super::native_tls::Error as TLSError;
use std::io::Error as OSFault;

///Unified Error Handling for IO errors and TLS errors
pub enum Fault {
	TLS(TLSError),
	OS(OSFault),
    None
}
impl Fault {
    #[inline(always)]
    pub fn exists(&self) -> bool {
        match self {
            &Fault::None => false,
            _ => true
        }
    }
}
impl From<OSFault> for Fault {

    ///From implemented to support ? notation
	fn from(x: OSFault) -> Fault {
		Fault::OS(x)
	}
}
impl From<TLSError> for Fault {

    ///From implemented to support ? notation
	fn from(x: TLSError) -> Fault {
		Fault::TLS(x)
	}
}
#[test]
fn test_fault() {
    use std::mem::{size_of,uninitialized,forget};
    assert_eq!( size_of::<Fault>(), 40);

    let x = unsafe{Fault::OS(uninitialized())};
    let y = unsafe{Fault::TLS(uninitialized())};
    let z = Fault::None;

    assert!( x.exists() );
    assert!( y.exists() );
    assert!( !z.exists());

    forget(x);
    forget(y);
    forget(z);
}
