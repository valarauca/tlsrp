
use super::conn::connection::Connection;
use super::conn::stream::{
    Stream,
    StreamType
};
use super::conn::fault::Fault;
use super::lock::{
    Lock,
    Locky
};
use super::mio::Token;
use std::cell::RefCell;
use std::sync::atomic::{
    AtomicPtr,
    Ordering
};
const ACQ: Ordering = Ordering::Acquire;

lazy_static! {
    static ref CONSLAB: AtomicPtr<Vec<Connection>> =
        AtomicPtr::new(
            Box::into_raw(
                Box::new(
                    Vec::with_capacity(10922))));
}

///function to get the raw pointer to the array
#[inline(always)]
fn raw_ptr<'a>() -> &'a mut Vec<Connection> {
    let ptr: *mut Vec<Connection> = CONSLAB.load(ACQ);
    unsafe {
        match ptr.as_mut() {
            Option::Some(x) => x,
            Option::None => unreachable!()
        }
    }
}


///Allocation connections, and fills buffer with empty connections
pub fn build_connections() {
    let mut ptr: &mut Vec<Connection>  = raw_ptr();
    for t in 0..10922 {
        let mut c = Connection::new();
        c.token = Token(t+1);
        ptr.push(c);
    }
}

///Attempts to access an index
pub enum Access<'a> {
    UnAllocated,
    Locked,
    Ok(&'a mut Connection),
}

///Give accecess to a connection. Or a connection pair
///if the connection has been paired.
pub fn get_connection<'a>( t: &Token) -> Access<'a> {
    let i = t.0-1;
    let mut slab: &'a mut Vec<Connection> = raw_ptr();
    let ptr: &'a mut Connection = &mut slab[i];
    if ! ptr.token_valid() {
        return Access::UnAllocated;
    }
    if ptr.try_lock() {
        return Access::Ok(ptr);
    }
    else {
        return Access::Locked;
    }
}

///Insert a stream. This spinlocks, as the lock MUST succeed. The event loop thread
///generally shouldn't be blocked, and it has a queue of de-allocated tokens so 
///this spinlock really should block _long_ if at all. Tokens _should not be_
///returned until their respective connections are closed.
pub fn assign_stream(t: &Token, x: Stream) -> Result<(),Stream> {
    let i: usize = t.0-1;
    let mut slab = raw_ptr(); 
    let mut ptr = &mut slab[i];
    ptr.spinlock();
    let ret_val = ptr.replace(x);
    ptr.unlock();
    ret_val
}
