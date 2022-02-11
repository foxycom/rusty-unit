use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

type Trace = Arc<Mutex<Vec<String>>>;

pub fn listen(port: u16, n_workers: usize) {
    let pool = ThreadPool::new(n_workers);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}", port = port)).unwrap();
    let traces = Arc::new(Mutex::new(Vec::new()));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let traces = traces.clone();
                pool.execute(move || {
                    handle_client(stream, traces);
                })
            }
            Err(e) => panic!("Encountered IO error: {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream, traces: Trace) {
    println!("Connected!");
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
                    println!("Requested data!");
                    //let serialized = serde_json::to_string(traces.as_ref()).unwrap() + "\n";
                    let serialized = traces.lock().as_ref().unwrap().join("\n") + "\n";
                    println!("Sending:\n{}", serialized);
                    stream.write(serialized.as_bytes()).unwrap();
                    //traces.lock().unwrap().clear();
                    stream.shutdown(Shutdown::Both);
                } else {
                    let mut lock = traces.lock();
                    let traces = lock.as_mut().unwrap();

                    // Remove last separator
                    msg.to_string()
                        .trim_end_matches(";")
                        .split(";")
                        .for_each(|line| {
                            println!("Pushing: {}", line);
                            traces.push(line.to_string());
                        });
                    //traces.entry(0).and_modify(|e| e.push(msg.to_string())).or_insert(vec![msg.to_string()]);
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!(
                    "An error occurred, terminating connection with {}\n{:?}",
                    stream.peer_addr().unwrap(),
                    e
                );
                stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }
}

fn main() {
    listen(3333, 10);
}
