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
    fn trace_fn(&self, name: &'static str, id: u64) { self.write(format!("root[{}, {}]", name, id)); }
    fn write(&self, message: String) { if let Some(sender) = &self.sender { sender.send(TestifyMessage::Line(message)).unwrap(); } }
    fn wait(&mut self) {
        if let Some(sender) = &self.sender { sender.send(TestifyMessage::Stop).unwrap(); }
        self.thread.take().unwrap().join().unwrap();
    }
}

use std::borrow::Borrow;

fn main() { LOGGER.with(|l| l.borrow().trace_fn("main", 1u64)); }

struct SomeStruct {
    a: u8,
    b: u8,
}

impl SomeStruct {
    pub fn new(a: u8, b: u8) -> SomeStruct {
        LOGGER.with(|l| l.borrow().trace_fn("new", 2u64));
        SomeStruct { a, b }
    }
    pub fn hello(&mut self, y: u8) {
        LOGGER.with(|l| l.borrow().trace_fn("hello", 5u64));
        if y < self.a {
            LOGGER.with(|l| l.borrow().trace_branch(3u64, 4u64, (self.a - y) as f64));
            self.b = y;
        } else {
            LOGGER.with(|l| l.borrow().trace_branch(4u64, 3u64, (y - self.a + 1u8) as f64));
            self.a = 3;
        }
    }
    pub fn test_reference(&mut self, x: u8) {
        LOGGER.with(|l| l.borrow().trace_fn("test_reference", 8u64));
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
        LOGGER.with(|l| l.borrow().trace_fn("new", 9u64));
        Rectangle { width, height }
    }
    pub fn width(&self) -> u8 {
        LOGGER.with(|l| l.borrow().trace_fn("width", 10u64));
        self.width
    }
}

struct AreaCalculator {}

impl AreaCalculator {
    pub fn new() -> Self {
        LOGGER.with(|l| l.borrow().trace_fn("new", 11u64));
        AreaCalculator {}
    }
    pub fn area_by_value(&self, rect: Rectangle) -> f64 {
        LOGGER.with(|l| l.borrow().trace_fn("area_by_value", 12u64));
        rect.height as f64 * rect.width as f64
    }
    pub fn area_by_ref(&self, rect: &Rectangle) -> f64 {
        LOGGER.with(|l| l.borrow().trace_fn("area_by_ref", 13u64));
        rect.height as f64 * rect.width as f64
    }
}

#[cfg(test)]
mod testify_tests {
    use super::*;

    #[test]
    fn testify_21() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(21u64));
        let mut somestruct_0 = SomeStruct::new(37u8, 214u8);
        somestruct_0.test_reference(218u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_22() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(22u64));
        let mut rectangle_0 = Rectangle::new(225u8, 35u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_value(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_23() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(23u64));
        let mut rectangle_0 = Rectangle::new(122u8, 10u8);
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_24() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(24u64));
        let mut rectangle_0 = Rectangle::new(125u8, 89u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_value(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_25() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(25u64));
        let mut somestruct_0 = SomeStruct::new(111u8, 166u8);
        somestruct_0.test_reference(158u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_26() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(26u64));
        let mut somestruct_0 = SomeStruct::new(27u8, 206u8);
        somestruct_0.test_reference(68u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_27() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(27u64));
        let mut rectangle_0 = Rectangle::new(139u8, 14u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_ref(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_28() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(28u64));
        let mut somestruct_0 = SomeStruct::new(111u8, 166u8);
        somestruct_0.test_reference(163u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_29() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(29u64));
        let mut somestruct_0 = SomeStruct::new(103u8, 186u8);
        somestruct_0.hello(106u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_30() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(30u64));
        let mut somestruct_0 = SomeStruct::new(16u8, 136u8);
        somestruct_0.test_reference(205u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_31() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(31u64));
        let mut somestruct_0 = SomeStruct::new(27u8, 134u8);
        somestruct_0.test_reference(86u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_32() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(32u64));
        let mut rectangle_0 = Rectangle::new(177u8, 65u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_value(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_33() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(33u64));
        let mut rectangle_0 = Rectangle::new(126u8, 100u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_ref(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_34() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(34u64));
        let mut somestruct_0 = SomeStruct::new(111u8, 166u8);
        somestruct_0.hello(250u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_35() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(35u64));
        let mut rectangle_0 = Rectangle::new(66u8, 120u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_ref(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_36() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(36u64));
        let mut rectangle_0 = Rectangle::new(110u8, 208u8);
        rectangle_0.width();
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_37() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(37u64));
        let mut rectangle_0 = Rectangle::new(22u8, 75u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_value(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_38() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(38u64));
        let mut somestruct_0 = SomeStruct::new(103u8, 186u8);
        somestruct_0.hello(95u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_39() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(39u64));
        let mut rectangle_0 = Rectangle::new(70u8, 224u8);
        let mut areacalculator_0 = AreaCalculator::new();
        areacalculator_0.area_by_value(rectangle_0);
        LOGGER.with(|l| l.borrow_mut().wait());
    }

    #[test]
    fn testify_40() {
        LOGGER.with(|l| l.borrow_mut().set_test_id(40u64));
        let mut somestruct_0 = SomeStruct::new(170u8, 25u8);
        somestruct_0.hello(195u8);
        LOGGER.with(|l| l.borrow_mut().wait());
    }
}