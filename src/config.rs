
use std::sync::atomic::{AtomicUsize,Ordering};

lazy_static! {
    static ref WORKERS: AtomicUsize = AtomicUsize::new(0);
}

///Set worker number
pub fn set_workers(x: usize) {
    WORKERS.store(x, Ordering::SeqCst);
}

///Get number of workers
pub fn get_workers() -> usize {
    WORKERS.load(Ordering::Relaxed)
}
