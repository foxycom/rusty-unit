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

use std::borrow::Borrow;

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

struct Rectangle {
    width: u8,
    height: u8,
}

impl Rectangle {
    pub fn new(width: u8, height: u8) -> Self {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("new"), 9u64));
        Rectangle { width, height }
    }
    pub fn width(&self) -> u8 {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("width"), 10u64));
        self.width
    }
}

struct AreaCalculator {}

impl AreaCalculator {
    pub fn new() -> Self {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("new"), 11u64));
        AreaCalculator {}
    }
    pub fn area(&self, rect: Rectangle) -> f64 {
        LOGGER.with(|l| l.borrow().trace_fn(String::from("area"), 12u64));
        rect.height as f64 * rect.width as f64
    }
}

#[cfg(test)]
mod testify_tests {
    use super::*;

    #[test]
    fn testify_61() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(61u64));
        let mut rectangle_0 = Rectangle::new(238u8, 135u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_62() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(62u64));
        let mut rectangle_0 = Rectangle::new(91u8, 146u8);
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_63() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(63u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_64() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(64u64));
        let mut rectangle_0 = Rectangle::new(91u8, 146u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_65() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(65u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_66() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(66u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_67() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(67u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_68() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(68u64));
        let mut rectangle_0 = Rectangle::new(238u8, 135u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_69() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(69u64));
        let mut somestruct_0 = SomeStruct::new(253u8, 48u8);
        somestruct_0.hello(115u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_70() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(70u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_71() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(71u64));
        let mut rectangle_1 = Rectangle::new(61u8, 235u8);
        let mut rectangle_0 = Rectangle::new(98u8, 158u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area(rectangle_0);
        areacalculator_0.area(rectangle_1);
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_72() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(72u64));
        let mut rectangle_0 = Rectangle::new(238u8, 135u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_73() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(73u64));
        let mut rectangle_1 = Rectangle::new(154u8, 108u8);
        let mut rectangle_0 = Rectangle::new(65u8, 5u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area(rectangle_1);
        areacalculator_0.area(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_74() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(74u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_75() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(75u64));
        let mut rectangle_1 = Rectangle::new(61u8, 235u8);
        let mut rectangle_0 = Rectangle::new(98u8, 158u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area(rectangle_0);
        rectangle_1.width();
        areacalculator_0.area(rectangle_1);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_76() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(76u64));
        let mut rectangle_1 = Rectangle::new(61u8, 235u8);
        let mut rectangle_0 = Rectangle::new(98u8, 158u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area(rectangle_0);
        areacalculator_0.area(rectangle_1);
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_77() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(77u64));
        let mut rectangle_1 = Rectangle::new(180u8, 211u8);
        let mut rectangle_0 = Rectangle::new(165u8, 65u8);
        let mut areacalculator_0 = AreaCalculator::new();
        rectangle_1.width();
        areacalculator_0.area(rectangle_1);
        areacalculator_0.area(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_78() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(78u64));
        let mut somestruct_0 = SomeStruct::new(158u8, 77u8);
        somestruct_0.test_reference(3u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_79() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(79u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_80() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(80u64));
        let mut rectangle_0 = Rectangle::new(95u8, 94u8);
        rectangle_0.width();
        rectangle_0.width();
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }
}