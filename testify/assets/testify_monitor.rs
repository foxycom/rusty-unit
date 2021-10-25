enum TestifyMessage {
    Stop,
    Line(String),
}
struct TestifyMonitor {
    sender: Option<std::sync::mpsc::Sender<TestifyMessage>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl TestifyMonitor {
    const TRACE_FILE: &'static str = "trace.txt";
    fn new() -> Self {
        TestifyMonitor {
            sender: None,
            thread: None,
        }
    }
    fn set_test_id(&mut self, id: u64) {
        let file = format!("traces/trace_{}.txt", id);
        let (tx, rx) = std::sync::mpsc::channel();
        let thread_handle = std::thread::spawn(move || {
            let trace_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file)
                .unwrap();
            let mut trace_file = std::io::LineWriter::new(trace_file);
            while let Ok(msg) = rx.recv() {
                match msg {
                    TestifyMessage::Stop => {
                        break;
                    }
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
    fn trace_branch(&self, visited_branch: u64, other_branch: u64, distance: f64) {
        self.write(format!(
            "branch[{}, {}, {}]",
            visited_branch, other_branch, distance
        ));
    }
    fn trace_fn(&self, name: &'static str, id: u64) {
        self.write(format!("root[{}, {}]", name, id));
    }
    fn write(&self, message: String) {
        if let Some(sender) = &self.sender {
            sender.send(TestifyMessage::Line(message)).unwrap();
        }
    }
    fn wait(&mut self) {
        if let Some(sender) = &self.sender {
            sender.send(TestifyMessage::Stop).unwrap();
        }
        self.thread.take().unwrap().join().unwrap();
    }
}