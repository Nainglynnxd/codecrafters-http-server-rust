#![allow(unused_imports, dead_code)]
mod http;
mod utils;

use anyhow::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use http::HttpRequest;
use std::{fs, fs::File, io::Write, thread};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use utils::*;
const ADDRESS: &str = "127.0.0.1:4221";
const HTTP_VERSION: &str = "HTTP/1.1";
const CRLF: &str = "\r\n";

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

fn handle_connection(mut _stream: TcpStream, request: &str) {
    let http_request = parse_request(request);
    println!("{:#?}", http_request.unwrap());
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
