#![allow(unused_imports, unused_variables)]
mod http;
mod utils;

use anyhow::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use http::HttpRequest;
use std::task::Wake;
use std::{fs, fs::File, io::Write, thread};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use utils::*;
const ADDRESS: &str = "127.0.0.1:4221";
// const CRLF: &str = "\r\n";

fn main() {
    let listener = TcpListener::bind(ADDRESS).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let mut buffer = [0_u8; 1024];
        match stream.peek(&mut buffer) {
            Ok(bytes) => {
                let request = String::from_utf8_lossy(&buffer[..bytes]).into_owned();
                handle_connection(stream, request.as_str());
            }
            Err(e) => eprintln!("Failed to read the stream: {e}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream, request: &str) {
    let mut response_headers = String::new();
    let response_body = Vec::new();
    let http_request = parse_request(request);
    let Request {
        method,
        route,
        version,
        user_agent,
        encoding,
    } = http_request.unwrap();

    match method.as_str() {
        "GET" => match route.as_str() {
            "/" => {
                response_headers.push_str(&format!("{} 200 OK\r\n\r\n", version));
            }
            _ => response_headers.push_str(&format!("{} 404 Not Found\r\n\r\n", version)),
        },
        "POST" => {}
        _ => {
            eprintln!("Invalid request method");
        }
    };

    let response = [response_headers.as_bytes(), &response_body].concat();
    stream.write_all(&response).unwrap();
}

fn parse_request(request: &str) -> Result<Request, Error> {
    let (method_line, headers) = request.split_once("\r\n").unwrap();

    let method_line: Vec<&str> = method_line.split_whitespace().collect();
    let (user_agent, encoding) = parse_header(headers);

    Ok(Request {
        method: String::from(method_line[0]),
        route: String::from(method_line[1]),
        version: String::from(method_line[2]),
        user_agent,
        encoding,
    })
}

fn parse_header(headers: &str) -> (Option<String>, Option<String>) {
    let mut lines = headers.lines();
    let user_agent = lines
        .find(|line| line.starts_with("User-Agent: "))
        .unwrap_or(&"")
        .replace("User-Agent: ", "");
    let encoding = lines
        .find(|line| line.starts_with("Accept-Encoding: "))
        .unwrap_or(&"")
        .replace("Accept-Encoding: ", "");

    (Some(user_agent), Some(encoding))
}

#[derive(Debug)]
struct Request {
    method: String,
    route: String,
    version: String,
    user_agent: Option<String>,
    encoding: Option<String>,
}
