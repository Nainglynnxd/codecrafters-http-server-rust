mod http;
mod utils;
use http::{extract_request_method_and_path, Request::*, StatusCode::*};
use utils::*;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted new connection");
                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            let response = match extract_request_method_and_path(request.lines().next()) {
                (GET, path) => match path {
                    "/" => OK.whole(),
                    p if p.starts_with("/echo/") => echo(path),
                    u if u.starts_with("/user-agent") => user_agent(&request),
                    f if f.starts_with("/files/") => file(path),
                    _ => NotFound.whole(),
                },
                (POST, path) => match path {
                    p if p.starts_with("/files/") => create_file(&request, path),
                    _ => NotFound.whole(),
                },
                (NONE, "") => String::new(),
                _ => String::new(),
            };
            stream.write(response.as_bytes()).unwrap();
        }
        Err(e) => eprintln!("Failed to read from connection: {}", e),
    }
}
