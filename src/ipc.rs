use super::slab::get_workerid;
use super::workerid::{
    WorkerID,
    get_id
};
use super::config::{
    get_workers,
    set_workers
};
use super::crossbeam::sync::SegQueue;
use super::mio::{
    Event,
    Token,
    Ready
};
use std::sync::atomic::{
    AtomicPtr,
    Ordering
};


///The Reqeuests a client can make to the event thread
///
/// - New asks for a new connection. The worker thread must track these requests. Eventually
/// this'll expand to support MULTIPLE items to forward too, currently there is only one
///
/// - Close. This signals the worker has CLOSED a connection, and it is returning the token
/// to the event loop.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Requests {
    New(usize),
    Close(Token)
}

///How the Main Event loop will see requests. It mearly bundles the orginal `Requests` enum with a 
///`WorkerID` so the main loop will know where it has to forward it's stuff back too.
#[derive(Clone,Copy,Debug)]
pub struct PresentRequests(pub WorkerID, pub Requests);

///Responses. What the event loop can say to a worker. Open shows that it has opened a new
///connection like the worker has requested. Event is a MIO event the worker thread in question
///has a lock on.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Events {
    Failure,
    Open(Token),
    Event(Event)
}

///A single workers IP channel. Handles the sending/recieving of messages to/from the worker thread
///to the main event thread
struct WorkerIPC {
    id: WorkerID,
    to: SegQueue<Requests>,
    from: SegQueue<Events>
}
impl WorkerIPC {
    
    ///Build a worker queue assocated with a worker
    fn new(id: usize) -> WorkerIPC {
        WorkerIPC {
            id: WorkerID(id),
            to: SegQueue::new(),
            from: SegQueue::new()
        }
    }
}

lazy_static! {
    static ref WORKER_BUS: AtomicPtr<Vec<WorkerIPC>> = 
        AtomicPtr::new(
            Box::into_raw(
                Box::new(
                    Vec::with_capacity(
                        get_workers()))));
}

///Gets a reference to the worker bus vector. This is an unsafe method since it allows for multiple
///mutable pointers to exist at once. 
#[inline(always)]
fn worker_bus<'a>() -> &'a mut Vec<WorkerIPC> {
    let mut ptr: *mut Vec<WorkerIPC> = WORKER_BUS.load(Ordering::Acquire);
    let mut reference: &'a mut Vec<WorkerIPC> = match unsafe{ ptr.as_mut() } {
        Option::Some(r) => r,
        _ => unreachable!()
    };
    reference
}
///Constructs the worker thread IPC.
pub fn build_ipc( worker_count: usize) {
    set_workers(worker_count);
    let mut ptr = worker_bus();
    for workerid in 1..worker_count {
        ptr.push(WorkerIPC::new(workerid));
    }
}

///Get messages from the event loop. Reads the events related to a specific worker thread. 
#[inline(never)]
pub fn my_events(v: &mut Vec<Events>) {
    let id = get_id();
    let i = id-1;
    let bus = worker_bus();
    let worker = &bus[i];
    if worker.id != WorkerID(id) {
        return;
    }
    while let Some(item) = worker.from.try_pop() {
        v.push(item);
    }
}

///Send Requests to the event loop
pub fn send_request(r: Requests) {
    let id = get_id();
    let i = id-1;
    let bus = worker_bus();
    let worker = &bus[i];
    if worker.id != WorkerID(id) {
        return;
    }
    worker.to.push(r);
}


///Get all requests from everything. This method should only be called by the event loop.
///It's goal is to collect _all_ of the events from each thread. This will likely fail as
///something will interupt it but next go-around it may catch those requests.
///
///Items are passed two a mutable vector so that the queue which can have a pre-allocated
///size. So reading events shouldn't cause undo memory pressure.
pub fn get_requests( v: &mut Vec<PresentRequests> ) {
    let bus = worker_bus();
    for worker in bus.iter() {
        while let Some(req) = worker.to.try_pop() {
            v.push(PresentRequests(worker.id,req));
        }
    }
}

///Filter Map events for the main thread
pub fn send_event(e: Event) -> Option<Event> {
    let t = e.token();
    let bus = worker_bus();
    match get_workerid(&t) {
        Option::None => return Some(e),
        Option::Some(id) => {
            let i = id.0-1;
            let worker = &bus[i];
            worker.from.push(Events::Event(e));
            None
        }
    }
}

///Futfil what a worker wanted the event loop to do
pub fn send_futfillment(w: WorkerID, e: Events) {
    let bus = worker_bus();
    let i = w.0-1;
    let worker = &bus[i];
    worker.from.push(e);
}
