
use std::sync::atomic::{AtomicUsize,Ordering};

thread_local! {
    static ID: AtomicUsize = AtomicUsize::new(0);
}

///Set worker id
pub fn set_id(x: usize) {
    ID.with(|a|{
        a.store(x,Ordering::Relaxed);
    });
}

///Get worker ID
#[inline(always)]
pub fn get_id() -> usize {
    ID.with(|a|{
        a.load(Ordering::Relaxed)
    })
}

///Identifies a worker
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct WorkerID(pub usize);


