#![allow(dead_code, unused_imports)]
mod http;
use http::extract_request_method_and_path;
use http::Request::{GET, NONE, POST};
use http::StatusCode::{Created, NotFound, OK};

use std::borrow::Cow;
use std::fs;
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
                    u if u.starts_with("/user-agent") => user_agent(request),
                    _ => NotFound.whole(),
                },
                (POST, "") => OK.whole(),
                (NONE, "") => String::new(),
                _ => String::new(),
            };
            stream.write(response.as_bytes()).unwrap();
        }
        Err(_) => {}
    }
}

fn echo(path: &str) -> String {
    let response = path.trim_start_matches("/echo/");
    format!(
        "{}Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        OK.part(),
        response.len(),
        response
    )
}

fn user_agent(path: Cow<str>) -> String {
    let mut agent = "";
    for header in path.lines() {
        if header.starts_with("User-Agent:") {
            agent = header.trim_start_matches("User-Agent:").trim();
            break;
        }
    }
    format!(
        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        OK.part(),
        agent.len(),
        agent
    )
}
