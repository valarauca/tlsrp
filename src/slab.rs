use super::workerid::WorkerID;
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
    for t in 10..10932 {
        let mut c = Connection::new();
        c.token = Token(t);
        ptr.push(c);
    }
}


///Ensure token to index is done consistently
#[inline(always)]
fn to_index(t: &Token) -> usize {
    let mut i = t.0;
    i - 10
}
#[test]
fn test_to_index() {
    for i in 10..10932 {
        assert!( to_index(&Token(i)) < 10922);
    }
}

///Attempts to access an index
pub enum Access<'a> {
    UnAllocated,
    Locked,
    Ok(&'a mut Connection),
}

///Get the ID assocated with a token. If there is no associated ID I.E.: It is special, or the
///connection is unallocated. It will return NONE.
#[inline(always)]
pub fn get_workerid(t: &Token) -> Option<WorkerID> {
    if t.0 < 10 {
        return None;
    }
    let i = to_index(t);
    let mut slab: &mut Vec<Connection> = raw_ptr();
    let ptr: &mut Connection = &mut slab[i];
    ptr.worker()
}

///Give accecess to a connection. Or a connection pair
///if the connection has been paired.
pub fn get_connection<'a>( t: &Token) -> Access<'a> {
    let i = to_index(t);
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
pub fn assign_stream(t: &Token, x: Stream, w: WorkerID) -> Result<(),Stream> {
    let i: usize = to_index(t);
    let mut slab = raw_ptr(); 
    let mut ptr = &mut slab[i];
    let ret_val = ptr.setup(x,w);
    ret_val
}
