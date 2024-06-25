mod http;
use http::Request::{GET, NONE, POST};
use http::StatusCode::{Created, NotFound, OK};

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
            let mut req_lines = request.lines();
            if let Some(request_line) = req_lines.next() {
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() == 3 {
                    let request_method = match parts[0] {
                        "GET" => GET,
                        "POST" => POST,
                        _ => NONE,
                    };

                    let path = parts[1];

                    let mut user_agent = "";
                    let mut content_length = 0;

                    for header in req_lines.clone() {
                        if header.starts_with("User-Agent:") {
                            user_agent = header.trim_start_matches("User-Agent:").trim();
                        } else if header.starts_with("Content-Length:") {
                            content_length = header
                                .trim_start_matches("Content-Length:")
                                .trim()
                                .parse()
                                .unwrap_or(0);
                        }
                    }

                    let response = match request_method {
                        GET => match path {
                            "/" => OK.default(),
                            f if f.starts_with("/files/") => {
                                let filename = &path[7..];
                                match fs::read(format!("/tmp/data/codecrafters.io/http-server-tester/{}", filename)) {
                                    Ok(contents) => {
                                        format!(
                                            "{}Content-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                                            OK.default(),
                                            contents.len(),
                                            String::from_utf8_lossy(&contents)
                                        )
                                    }
                                    Err(_) => NotFound.default()
                                }
                            }
                            p if p.starts_with("/echo/") => {
                                let response_body = path.trim_start_matches("/echo/");
                                format!("{}Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                    OK.default(),
                                    response_body.len(),
                                    response_body
                                )
                            }
                            "/user-agent" => format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                user_agent.len(),
                                user_agent
                            ),

                            _ => NotFound.default()
                        },
                        POST => match path {
                            p if p.starts_with("/files/") => {
                                let filename = &path[7..];

                                let mut headers_done = false;
                                let mut body_start_index = 0;

                                for (i, line) in request.lines().enumerate() {
                                    if line.is_empty() {
                                        headers_done = true;
                                        body_start_index = i + 1;
                                        break;
                                    }
                                }

                                if headers_done {
                                    let body = &buffer[size - content_length..size];
                                    let file_path = format!("/tmp/data/codecrafters.io/http-server-tester/{}", filename);
                                    match fs::write(&file_path, body) {
                                        Ok(_) => Created.default(),
                                        Err(e) => {
                                            eprintln!("Failed to write to file: {}", e);
                                            "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_owned()
                                        }
                                    }
                                } else {
                                    "HTTP/1.1 400 Bad Request\r\n\r\n".to_owned()
                                }
                            }

                            _ => NotFound.default()
                        },
                        _ => "HTTP/1.1 405 Method Not Allowed\r\n\r\n".to_owned(),
                    };

                    if !response.is_empty() {
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
