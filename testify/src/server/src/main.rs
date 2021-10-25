use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use threadpool::ThreadPool;
use std::sync::{Arc, Mutex};
use std::io::ErrorKind;

type Trace = Arc<Mutex<HashMap<usize, Vec<String>>>>;

pub fn listen(port: u16, n_workers: usize) {
    let pool = ThreadPool::new(n_workers);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}", port=port)).unwrap();
    let traces = Arc::new(Mutex::new(HashMap::new()));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connected");
                let traces = traces.clone();
                pool.execute(move || {
                    handle_client(stream, traces);
                })
            }
            Err(e) => panic!("Encountered IO error: {}", e)
        }
    }
}

fn handle_client(mut stream: TcpStream, traces: Trace) {
    let mut data = [0 as u8; 1024]; // using 1024 byte buffer
    loop {
        match stream.read(&mut data) {
            Ok(size) => {
                if size == 0 {
                    // Connection closed
                    break;
                }
                let msg = String::from_utf8_lossy(&data[0..size]);
                if msg.starts_with("get") {
                    let serialized = serde_json::to_string(traces.as_ref()).unwrap();
                    traces.lock().unwrap().clear();
                    stream.write(serialized.as_bytes()).unwrap();
                } else {
                    let mut lock = traces.lock();
                    let traces = lock.as_mut().unwrap();
                    traces.entry(0).and_modify(|e| e.push(msg.to_string())).or_insert(vec![msg.to_string()]);
                }
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("An error occurred, terminating connection with {}\n{:?}", stream.peer_addr().unwrap(), e);
                stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }
}

fn main() {
    listen(3333, 10);
}