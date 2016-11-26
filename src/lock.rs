

use std::sync::atomic::{AtomicUsize,Ordering};
const SEQ: Ordering = Ordering::SeqCst;
const REL: Ordering = Ordering::Relaxed;


///An Abstract trait that allows for things to act like locks
pub trait Locky {
    fn try_lock(&self) -> bool;
    fn unlock(&self);
    fn spinlock(&self);
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
}

impl Locky for Lock {
    ///Attempt to lock. Return's true if the lock succeeded
    #[inline(always)]
    fn try_lock(&self) -> bool {
        self.locky.compare_and_swap(0,1,SEQ) == 0
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
        while self.locky.compare_and_swap(0,1,SEQ) == 1 {
            continue
        }
    }
}
#[test]
fn test_lock() {
    let l = Lock::new();
    assert!( ::std::mem::size_of::<Lock>() == 64);
    assert!( l.locky.load(REL) == 0usize);
    assert!( l.try_lock() );
    assert!( l.locky.load(REL) == 1usize);
    l.unlock();
    assert!( l.locky.load(REL) == 0usize);
    l.spinlock();
    assert!( l.locky.load(REL) == 1usize);
}
