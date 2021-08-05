use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::Read;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use crate::parser::TraceParser;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    Start(Job),
    Stop,
}

enum TraceMessage {
    Line(String)
}

pub struct Server {
    port: u16,
    thread: Option<JoinHandle<()>>,
    traces: Arc<Mutex<HashMap<u64, HashMap<u64, f64>>>>,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server {
            port,
            thread: None,
            traces: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn traces(&self) -> &Arc<Mutex<HashMap<u64, HashMap<u64, f64>>>> {
        &self.traces
    }

    pub fn listen(&mut self) {
        let addr = format!("127.0.0.1:{}", self.port);
        let trace = self.traces.clone();
        let thread = thread::spawn(move || {
            let listener = TcpListener::bind(addr).unwrap();
            let pool = ThreadPool::new(8);

            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let trace = trace.clone();
                pool.execute(move || {
                    let mut buffer = [0; 1024];
                    let bytes_read = stream.read(&mut buffer).unwrap();
                    let line = String::from_utf8_lossy(&buffer[..bytes_read]);

                    for part in line.split("||") {
                        if part.is_empty() {
                            break;
                        }

                        // Parse the trace line
                        let data = TraceParser::parse_line(part).unwrap();
                        trace.lock().unwrap()
                            .entry(data.test_id())
                            .and_modify(|e| {
                                e.insert(data.branch_id(), 0.0);
                            })
                            .or_insert({
                                let mut m = HashMap::new();
                                m.insert(data.branch_id(), 0.0);
                                m
                            });
                        if let Some(other_branch) = data.other_branch_id() {
                            let dist = data.distance().unwrap();
                            trace.lock().unwrap()
                                .entry(data.test_id())
                                .and_modify(|e| {
                                    e.insert(other_branch, dist);
                                })
                                .or_insert({
                                    let mut m = HashMap::new();
                                    m.insert(other_branch, dist);
                                    m
                                });
                        }
                    }
                });
            }
        });
        self.thread = Some(thread);
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let job = Box::new(f);
        self.sender.send(Message::Start(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Stop).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,

}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::Start(job) => {
                    job();
                }
                Message::Stop => {
                    break;
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}

