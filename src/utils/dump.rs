mod http;
mod utils;

use anyhow::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use http::HttpRequest;
use std::net::{TcpListener, TcpStream};
use std::{fs, fs::File, io::Write, thread};
use utils::*;
const ADDRESS: &str = "127.0.0.1:4221";
const HTTP_VERSION: &str = "HTTP/1.1";
const CRLF: &str = "\r\n";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let dir = if args.len() > 2 {
        args.windows(2)
            .find(|window| window[0] == "--directory") // (--directory /[server directory]/)
            .map_or(String::new(), |window| window[1].to_owned())
    } else {
        String::new()
    };

    let listener = TcpListener::bind(ADDRESS).unwrap();

    println!("Server is running on http://{}", ADDRESS);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory = dir.clone();

                thread::spawn(move || {
                    let mut request = [0_u8; 1024];

                    match stream.peek(&mut request) {
                        Ok(bytes) => {
                            let request_string =
                                String::from_utf8_lossy(&request[..bytes]).into_owned();
                            handle_connection(stream, &request_string, directory);
                        }
                        Err(e) => eprintln!("Failed to read from the stream: {e}"),
                    }
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, request_string: &str, directory: String) {
    println!("{}", request_string);

    let http_request = match parse_request(request_string) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Failed to parse request: {e}");
            return;
        }
    };

    let (request_line, remaining_lines) = match request_string.split_once(CRLF) {
        Some((req_line, remaining)) => (req_line, remaining),
        None => {
            eprintln!("Incorrect request: {}", request_string);
            return;
        }
    };

    let (req_method, the_rest) = match request_line.split_once(" ") {
        Some((method, the_rest)) => (method, the_rest),
        None => {
            eprintln!("Incorrect request line: {}", request_line);
            return;
        }
    };

    // status lines
    let ok = format!("{} 200 OK\r\n", HTTP_VERSION);
    let not_found = format!("{} 404 Not Found\r\n\r\n", HTTP_VERSION);
    let mut response_headers = String::new();
    let mut response_body = Vec::new();

    match req_method {
        "GET" => match http_request.path.as_str() {
            "/" => {
                response_headers.push_str(&ok);
                response_headers.push_str(CRLF);
            }
            path if path.starts_with("/echo/") => {
                let body = path.replace("/echo/", "");
                response_headers.push_str(&ok);
                response_headers.push_str(&String::from("Content-Type: text/plain\r\n"));
                if let Some(encoding) = http_request.valid_encoding {
                    if encoding.contains("gzip") {
                        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                        encoder.write_all(body.as_bytes()).unwrap();
                        let compressed_body = encoder.finish().unwrap();
                        response_headers.push_str("Content-Encoding: gzip\r\n");
                        response_headers
                            .push_str(&format!("Content-Length: {}\r\n", compressed_body.len()));

                        response_body = compressed_body;
                    } else {
                        response_body = body.into_bytes();
                        response_headers
                            .push_str(&format!("Content-Length: {}\r\n", response_body.len()));
                    }
                } else {
                    response_body = body.into_bytes();
                    response_headers
                        .push_str(&format!("Content-Length: {}\r\n", response_body.len()));
                }
                response_headers.push_str(CRLF);
            }
            "/user-agent" => {
                let body = http_request.user_agent.unwrap_or_default();
                response_headers.push_str(&ok);
                response_headers.push_str(&format!(
                    "Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                ));
                response_body = body.into_bytes();
            }
            path if path.starts_with("/files/") => {
                let filename = path.replace("/files/", "");
                let mut file_path = directory.to_owned();
                if !file_path.ends_with("/") {
                    file_path.push('/');
                }
                file_path.push_str(&filename);
                if file_path_exists(&file_path) {
                    match fs::read(&file_path) {
                        Ok(content) => {
                            response_headers.push_str(&ok);
                            response_headers.push_str(&format!("Content-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                            content.len()));
                            response_body = content;
                        }
                        Err(e) => {
                            eprintln!("Failed to read file: {}", e);
                            response_headers.push_str(&not_found);
                        }
                    }
                } else {
                    eprintln!("File does not exist at path: {}", file_path);
                    response_headers.push_str(&not_found);
                }
            }
            _ => {
                eprintln!("Unknown GET path: {}", http_request.path);
                response_headers.push_str(&not_found)
            }
        },
        "POST" => {
            let body = remaining_lines.split_once("\r\n\r\n").unwrap().1;
            let filename = the_rest.split_once(" ").unwrap().0;
            let filename = filename.strip_prefix("/files/").unwrap();
            let mut filepath = directory.to_owned();
            if !filepath.ends_with("/") {
                filepath.push('/');
            }
            filepath.push_str(filename);
            let mut file = match File::create(&filepath) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to create file: {}", e);
                    return;
                }
            };
            if let Err(e) = file.write_all(body.as_bytes()) {
                eprintln!("Failed to write to file: {e}");
                return;
            }
            response_headers.push_str(
                "HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\nContent-Length: 0\r\n\r\n",
            );
        }
        _ => {
            println!("Unknown method: {}", req_method);
            response_headers.push_str(&not_found);
        }
    }

    let response = [response_headers.as_bytes(), &response_body].concat();
    if let Err(e) = stream.write_all(&response) {
        println!("Failed to send response: {}", e);
    }
}
