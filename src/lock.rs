

use super::workerid::{WorkerID,get_id};
use std::sync::atomic::{AtomicUsize,Ordering};
const SEQ: Ordering = Ordering::SeqCst;
const REL: Ordering = Ordering::Relaxed;


///An Abstract trait that allows for things to act like locks
pub trait Locky {
    fn try_lock(&self) -> bool;
    fn unlock(&self);
    fn spinlock(&self);
    fn worker(&self) -> Option<WorkerID>;
}

///A simple lock primative. Takes up 64bytes, a whole cache line on most AMD64 systems.
#[repr(C)]
#[allow(dead_code)]
pub struct Lock {
    locky: AtomicUsize,
    pad: [u64;7]
}
impl Lock {

    ///Create a new lock. In an unlocked state
    pub fn new() -> Lock {
        Lock {
            locky: AtomicUsize::new(0),
            pad: [0u64;7]
        }
    }

    ///Create with a worker in mind
    pub fn give(w: WorkerID) -> Lock {
        Lock {
            locky: AtomicUsize::new(w.0),
            pad: [0u64;7]
        }
    }

    ///manually set the lock
    pub fn manual_set(&self, w: WorkerID) {
        self.locky.store(w.0,REL);
    }
}

impl Locky for Lock {
    ///Get the worker associated with a lock
    #[inline(always)]
    fn worker(&self) -> Option<WorkerID> {
        let i = self.locky.load(REL);
        if i == 0 {
            None
        } else {
            Some(WorkerID(i))
        }
    }
    
    ///Attempt to lock. Return's true if the lock succeeded
    #[inline(always)]
    fn try_lock(&self) -> bool {
        let id = get_id();
        self.locky.compare_and_swap(0,id,SEQ) == 0
    }

    ///Unlock the lock.
    ///
    ///#WARNING
    ///
    ///This will unlock the lock U N C O N D I T I O N A L L Y. For
    ///E V E R Y O N E. So only call this function if you have first
    ///acquired.
    #[inline(always)]
    fn unlock(&self) {
        self.locky.store(0,REL);
    }

    ///Spinlock. This function will block until it acquires a lock.
    ///It may dead lock your program. Use with caution.
    #[inline(always)]
    fn spinlock(&self) {
        let id = get_id();
        while self.locky.compare_and_swap(0,id,SEQ) != 0 {
            continue
        }
    }
}
#[test]
fn test_lock() {
    use super::workerid::set_id;
    set_id(1);
    let l = Lock::new();
    assert_eq!( l.worker(), None);
    assert!( ::std::mem::size_of::<Lock>() == 64);
    assert!( l.locky.load(REL) == 0usize);
    assert!( l.try_lock() );
    assert!( l.locky.load(REL) == 1usize);
    assert_eq!( l.worker(), Some(WorkerID(1)));
    l.unlock();
    assert!( l.locky.load(REL) == 0usize);
    l.spinlock();
    assert!( l.locky.load(REL) == 1usize);
}
