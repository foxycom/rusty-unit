use std::fmt::Write as FmtWrite;
use std::io::Write; thread_local! { pub static TEST_ID : std :: cell :: RefCell < u64 > = std :: cell :: RefCell :: new (0) ; static LOGGER : std :: cell :: RefCell < TestifyMonitor > = std :: cell :: RefCell :: new (TestifyMonitor :: new ()) ; } enum TestifyMessage { Stop, Line(String) }

struct TestifyMonitor {
    sender: Option<std::sync::mpsc::Sender<TestifyMessage>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl TestifyMonitor {
    const TRACE_FILE: &'static str = "trace.txt";
    fn new() -> Self { TestifyMonitor { sender: None, thread: None } }
    fn set_test_id(&mut self, id: u64) {
        let file = format!("traces/trace_{}.txt", id);
        let (tx, rx) = std::sync::mpsc::channel();
        let thread_handle = std::thread::spawn(move || {
            let trace_file = std::fs::OpenOptions::new().create(true).append(true).open(file).unwrap();
            let mut trace_file = std::io::LineWriter::new(trace_file);
            while let Ok(msg) = rx.recv() {
                match msg {
                    TestifyMessage::Stop => { break; }
                    TestifyMessage::Line(line) => {
                        let mut s = String::new();
                        writeln!(s, "{}", line);
                        trace_file.write_all(s.as_bytes()).unwrap();
                    }
                }
            }
        });
        self.sender = Some(tx);
        self.thread = Some(thread_handle);
    }
    fn trace_branch(&self, visited_branch: u64, other_branch: u64, distance: f64) { self.write(format!("branch[{}, {}, {}]", visited_branch, other_branch, distance)); }
    fn trace_fn(&self, name: String, id: u64) { self.write(format!("root[{}, {}]", name, id)); }
    fn write(&self, message: String) { if let Some(sender) = &self.sender { sender.send(TestifyMessage::Line(message)).unwrap(); } }
    fn wait(&mut self) {
        if let Some(sender) = &self.sender { sender.send(TestifyMessage::Stop).unwrap(); }
        self.thread.take().unwrap().join().unwrap();
    }
}

fn main() { LOGGER.with(|l| l.borrow().trace_fn(String::from("main"), 1u64)); }

pub fn hello(y: u8) {
    LOGGER.with(|l| l.borrow().trace_fn(String::from("hello"), 4u64));
    let mut x = 3;
    if y < x {
        LOGGER.with(|l| l.borrow().trace_branch(2u64, 3u64, (x - y) as f64));
        x = y;
    } else {
        LOGGER.with(|l| l.borrow().trace_branch(3u64, 2u64, (y - x + 1u8) as f64));
        x = 3;
    }
}

pub fn test_reference(x: u8) {
    LOGGER.with(|l| l.borrow().trace_fn(String::from("test_reference"), 7u64));
    if x == 3 {
        LOGGER.with(|l| l.borrow().trace_branch(5u64, 6u64, 1.0));
        let a = 4;
        let x = a;
    } else { LOGGER.with(|l| l.borrow().trace_branch(6u64, 5u64, ((x - 3) as f64).abs())); }
}

#[cfg(test)]
mod testify_tests {}