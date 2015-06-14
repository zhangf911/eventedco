use std::cell::UnsafeCell;
use std::io;

use mio::util::Slab;
use mio::{Handler, EventLoop, Token, ReadHint, Interest, Evented, PollOpt};

use coroutine::{Coroutine, Handle};

thread_local!(static PROCESSOR: UnsafeCell<Processor> = UnsafeCell::new(Processor::new()));

pub struct Processor {
    event_loop: EventLoop<IoHandler>,
    handler: IoHandler,
}

impl Processor {
    fn new() -> Processor {
        Processor {
            event_loop: EventLoop::new().unwrap(),
            handler: IoHandler::new(1024),
        }
    }

    pub fn current() -> &'static mut Processor {
        PROCESSOR.with(|p| unsafe {
            &mut *p.get()
        })
    }

    pub fn register<E: Evented>(&mut self, io: &E, inst: Interest) -> io::Result<()> {
        let cur_hdl = Coroutine::current().clone();
        let token = self.handler.slabs.insert(cur_hdl).unwrap();
        try!(self.event_loop.register_opt(io, token, inst, PollOpt::oneshot()));
        Coroutine::block();
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.event_loop.run(&mut self.handler)
    }
}

struct IoHandler {
    slabs: Slab<Handle>,
}

impl IoHandler {
    fn new(size: usize) -> IoHandler {
        IoHandler {
            slabs: Slab::new(size),
        }
    }
}

impl Handler for IoHandler {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, _: &mut EventLoop<IoHandler>, token: Token, _: ReadHint) {
        match self.slabs.remove(token) {
            Some(hdl) => {
                let res = hdl.resume();
                debug!("Readable: Resume resule: {:?}", res);
            },
            None => {}
        }
    }

    fn writable(&mut self, _: &mut EventLoop<IoHandler>, token: Token) {
        match self.slabs.remove(token) {
            Some(hdl) => {
                let res = hdl.resume();
                debug!("Readable: Resume resule: {:?}", res);
            },
            None => {}
        }
    }
}
