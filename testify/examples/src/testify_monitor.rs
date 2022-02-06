use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::str::from_utf8;

thread_local! {
    static MONITOR: Rc<RefCell<Monitor>> = Rc::new(RefCell::new(Monitor::new()));
}

struct Monitor {
    connection: TcpStream,
    test_id: u64
}

const ROOT_BRANCH: &'static str = "root[{}, {}]";
const BRANCH: &'static str = "branch[{}, {}, {}]";

impl Monitor {
    pub fn set_test_id(&mut self, test_id: u64) {
        self.test_id = test_id;
    }

    pub fn trace_fn(&mut self, name: &str, id: u64) {
        let msg = format!("root[{}, {}]", name, id);
        self.send(&msg);
    }
    pub fn trace_branch(&mut self, self_branch: u64, other_branch: u64, dist: f64) {
        let msg = format!("branch[{}, {}, {}]", self_branch, other_branch, dist);
        self.send(&msg);
    }

    fn send(&mut self, msg: &str) {
        self.connection.write(msg.as_bytes()).unwrap();
    }

    fn new() -> Self {
        let connection = match TcpStream::connect("localhost:3333") {
            Ok(stream) => stream,
            Err(e) => {
                println!("Failed to connect: {}", e);
                panic!()
            }
        };
        Monitor { connection, test_id: 0 }
    }
}

pub fn trace_fn(name: &str, id: u64) {
    MONITOR.with(|m| m.borrow_mut().trace_fn(name, id));
}

pub fn trace_branch(self_branch: u64, other_branch: u64, dist: f64) {
    MONITOR.with(|m| m.borrow_mut().trace_branch(self_branch, other_branch, dist));
}