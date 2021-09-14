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

struct SomeStruct {
    a: u8,
    b: u8,
}

impl SomeStruct {
    pub fn new(a: u8, b: u8) -> SomeStruct {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("new"), 2u64));
        SomeStruct { a, b }
    }
    pub fn hello(&mut self, y: u8) {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("hello"), 5u64));
        if y < self.a {
            LOGGER.with(|l| l.borrow().trace_branch(3u64, 4u64, (self.a - y) as f64));
            self.b = y;
        } else {
            LOGGER.with(|l| l.borrow().trace_branch(4u64, 3u64, (y - self.a + 1u8) as f64));
            self.a = 3;
        }
    }
    pub fn test_reference(&mut self, x: u8) {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("test_reference"), 8u64));
        if x == 3 {
            LOGGER.with(|l| l.borrow().trace_branch(6u64, 7u64, 1.0));
            self.a = 4;
        } else { LOGGER.with(|l| l.borrow().trace_branch(7u64, 6u64, ((x - 3) as f64).abs())); }
    }
}

#[cfg(test)]
mod testify_tests {
    use super::*;

    #[test]
    fn testify_201() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(201u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(125u8);
        SomeStruct_0.test_reference(167u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_202() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(202u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.hello(124u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(119u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_203() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(203u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(72u8);
        SomeStruct_0.test_reference(167u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_204() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(204u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.hello(248u8);
        SomeStruct_0.hello(207u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_206() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(206u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(219u8);
        SomeStruct_0.test_reference(76u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(53u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_208() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(208u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.hello(124u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(199u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_210() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(210u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.test_reference(63u8);
        SomeStruct_0.hello(120u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_211() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(211u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(167u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_212() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(212u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.test_reference(44u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(70u8);
        SomeStruct_0.hello(231u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_213() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(213u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(76u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(53u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_214() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(214u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.hello(248u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(165u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_215() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(215u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.hello(76u8);
        SomeStruct_0.hello(124u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_216() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(216u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.test_reference(80u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.test_reference(145u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_218() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(218u64));
        let mut SomeStruct_0 = SomeStruct::new(98u8, 127u8);
        SomeStruct_0.hello(52u8);
        SomeStruct_0.test_reference(252u8);
        SomeStruct_0.test_reference(66u8);
        SomeStruct_0.hello(64u8);
        SomeStruct_0.test_reference(5u8);
        SomeStruct_0.hello(198u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_219() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(219u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(232u8);
        SomeStruct_0.test_reference(70u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(235u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_220() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(220u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.test_reference(89u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_181() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(181u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(167u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_182() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(182u64));
        let mut SomeStruct_0 = SomeStruct::new(165u8, 105u8);
        SomeStruct_0.test_reference(80u8);
        SomeStruct_0.hello(12u8);
        SomeStruct_0.hello(178u8);
        SomeStruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_184() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(184u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.hello(124u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_185() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(185u64));
        let mut SomeStruct_0 = SomeStruct::new(160u8, 222u8);
        SomeStruct_0.test_reference(3u8);
        SomeStruct_0.hello(248u8);
        SomeStruct_0.test_reference(97u8);
        SomeStruct_0.hello(213u8);
        SomeStruct_0.hello(3u8);
        SomeStruct_0.test_reference(56u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }
}