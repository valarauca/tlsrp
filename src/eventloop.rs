use super::slab::{
    build_connections,
    assign_stream,
};
use super::workerid::WorkerID;
use super::ipc::{
    PresentRequests,
    get_requests,
    Requests,
    Events,
    build_ipc,
    send_event,
    send_futfillment,
};
use super::mio::{
    PollOpt,
    Poll,
    Ready,
    Events as EventBuff,
    Token,
};
use super::mio::tcp::{
    TcpListener,
    TcpStream,
};
use super::mio::deprecated::{
    UnixStream,
    UnixListener,
};
use super::conn::stream::Stream;
use std::str::FromStr;
use std::path::PathBuf;
use std::io::prelude::*;
use std::net::SocketAddr;
use super::conn::fault::Fault;
use super::native_tls::TlsAcceptor;
use std::collections::BinaryHeap;

///What we forward connections too
pub enum Forward {
    Unix(PathBuf),
    Network(SocketAddr)
}
impl Forward {

    ///Build a forwarding address. If it returns NONE then a invalid socket address was given AND
    ///in the event it WAS a path. The file path points to nothing
    pub fn build(s: &str) -> Option<Forward> {
        match SocketAddr::from_str(s) {
            Ok(x) => Some(Forward::Network(x)),
            Err(_) => {
                let p = PathBuf::from(s);
                if p.exists() {
                    Some(Forward::Unix(p))
                } else {
                    None
                }
            }
        }
    }

    ///Setup up a new connection
    pub fn connect(&self, p: &Poll, t: Token) -> Result<Stream,Fault> {
        match self {
            &Forward::Network(ref socket) => {
                let tcp = TcpStream::connect(socket)?;
                let stream = Stream::create_tcp(tcp,p,t)?;
                Ok(stream)
            },
            &Forward::Unix(ref path) => {
                let unix = UnixStream::connect(path)?;
                let stream = Stream::create_unix(unix,p,t)?;
                Ok(stream)
            }
        }
    }
}

const SOCK: Token = Token(0);

///Intentionally returns 1 > index to convert for WorkID as 0 means unallocated/unused
#[inline(always)]
fn find_smallest_index(v: &[usize] ) -> usize {
    let mut value = ::std::usize::MAX;
    let mut index = 0;
    for i in 0..v.len() {
        let temp = v[i];
        if temp < value {
            index = i;
            value = temp;
        }
    }
    index+1
}
#[test]
fn test_smallest_index() {
    let x: [usize;5] = [0,15,25,4,20];
    assert_eq!(1, find_smallest_index(&x));
    let x: [usize;5] = [100,15,25,4,20];
    assert_eq!(4, find_smallest_index(&x));
}

///Construct the main loop
pub fn main_loop(
    to: Vec<Forward>,
    accept: TlsAcceptor,
    listen: SocketAddr,
    cli: PathBuf,
    worker_count: usize
) -> Result<(),Fault>
{
    //set up background memory
    build_ipc(worker_count);
    build_connections();

    //keep track of worker thread workload
    let mut workload = Vec::<usize>::with_capacity(worker_count);
    for _ in 0..worker_count {
        workload.push(0);
    }

    //allocate room to hear about worker requests
    let mut incoming = Vec::<PresentRequests>::with_capacity(256);
    
    //allocate room for events
    let mut events = EventBuff::with_capacity(256);
    
    //allocate unused tokens
    let mut heap = BinaryHeap::<Token>::with_capacity(10922);
    for t in 10..1932 {
        heap.push(Token(t));
    }

    //build the epoll
    let poll = Poll::new()?;

    //listen for CLI args
    let cmd = UnixListener::bind(&cli)?;
    
    //listen for connects
    let sock = TcpListener::bind(&listen)?;
   
    //register listeners
    poll.register(&sock,SOCK,Ready::readable(),PollOpt::level())?;

    //main loop
    loop {
    
        //listen for events
        poll.poll(&mut events, None);

        //loop over events
        for event in events.iter().filter_map(send_event) {
            match event.token() {

                //extern listener events
                SOCK => {

                    //see if there is a token avalible
                    let new_token = match heap.pop() {
                        Option::None => continue,
                        Option::Some(t) => t
                    };

                    //attempt to get the connection
                    let new_conn = match sock.accept() {
                        Ok((x,_)) => x,
                        Err(e) => {
                            //TODO LOGGING
                            heap.push(new_token);
                            continue;
                        }
                    };

                    //start TLS handshake + register with epoll
                    let new_stream = match Stream::create_tls(new_conn,&poll, new_token, &accept) {
                        Ok(x) => x,
                        Err(e) => {
                            //TODO logging
                            heap.push(new_token);
                            continue;
                        }
                    };
                    
                    //assign to worker with lightest load
                    let i = find_smallest_index(workload.as_slice());
                    let w = WorkerID(i);
                
                    //lock stream so only 1 thing can read it
                    match assign_stream(&new_token, new_stream, w) {
                        Ok(_) => { },
                        Err(e) => {
                            //TODO logging
                            heap.push(new_token);
                            continue;
                        }
                    };
                    //alert worker of new connection
                    send_futfillment(w, Events::Open(new_token));

                    //mark the worker has a larger load
                    workload[i-1] += 1;
                },
                _ => {
                    //TODO logging
                    //these events shouldn't happen
                }
            };
        }

        //read messages from other threads
        get_requests(&mut incoming);
        for req in incoming.iter() {
            match &req.1 {
                //worker wants a new connection
                &Requests::New(i) => {
                    if i >= to.len() {
                        //TODO log this event
                        continue;
                    }
                    //get a token
                    let t = match heap.pop() {
                        Option::None => unreachable!(),
                        Option::Some(x) => x,
                    };
                    //connect to the stream (and register it)
                    let stream = match to[i].connect(&poll,t) {
                        Ok(s) => s,
                        Err(e) => {
                            //TODO log this event
                            heap.push(t);
                            send_futfillment(req.0,Events::Failure);
                            continue;
                        }
                    };
                    //assign it to a client
                    match assign_stream(&t,stream,req.0) {
                        Ok(_) => { },
                        Err(e) => { 
                            //TODO log this event
                            heap.push(t);
                            send_futfillment(req.0,Events::Failure);
                            continue;
                        }
                    };
                    //alert client work is done
                    send_futfillment(req.0,Events::Open(t));
                },
                //worker has closed a connection
                &Requests::Close(t) => {
                    heap.push(t);
                    let w = req.0;
                    let i = w.0-1;
                    workload[i] -= 1;
                    //no futfilment sent for closing
                }
            };
        }
        incoming.clear();
    }
}
