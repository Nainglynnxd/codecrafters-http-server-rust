mod http;
mod utils;

use http::{Request, Response};
use std::net::TcpListener;
use std::thread;
use utils::*;
const ADDRESS: &str = "127.0.0.1:4221";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let dir = if args.len() > 2 {
        args.windows(2)
            .find(|window| window[0] == "--directory")
            .map_or(String::new(), |window| window[1].to_owned())
    } else {
        String::new()
    };

    let listener = TcpListener::bind(ADDRESS).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let directory = dir.clone();
        thread::spawn(|| {
            let mut buffer = [0_u8; 1024];
            match stream.peek(&mut buffer) {
                Ok(bytes) => {
                    let request = String::from_utf8_lossy(&buffer[..bytes]).into_owned();
                    handle_connection(stream, request.as_str(), directory);
                }
                Err(e) => eprintln!("Failed to read the stream: {e}"),
            }
        });
    }
}
