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
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\n\r\n";
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer);

            let mut req_lines = request.lines();
            if let Some(request_line) = req_lines.next() {
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() == 3 {
                    let request_method = parts[0];
                    let path = parts[1];

                    let mut user_agent = "";

                    for header in req_lines {
                        if header.starts_with("User-Agent:") {
                            user_agent = header.trim_start_matches("User-Agent:").trim();
                            break;
                        }
                    }

                    let response = if request_method == "GET" {
                        match path {
                            "/" => "HTTP/1.1 200 OK\r\n\r\n".to_owned(),
                            f if f.starts_with("/files/") => {
                                let filename = &path[7..];
                                match fs::read(format!("/tmp/data/codecrafters.io/http-server-tester/{}", filename)) {
                                    Ok(contents) => {
                                        format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                                            contents.len(),
                                            String::from_utf8_lossy(&contents)
                                        )
                                    }
                                    Err(_) => NOT_FOUND.to_owned(),
                                }
                            }
                            p if p.starts_with("/echo/") => {
                                let response_body = path.trim_start_matches("/echo/");
                                format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                    response_body.len(),
                                    response_body
                                )
                            }
                            "/user-agent" => format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                user_agent.len(),
                                user_agent
                            ),
                            _ => NOT_FOUND.to_owned(),
                        }
                    } else {
                        "HTTP/1.1 405 Method Not Allowed\r\n\r\n".to_owned()
                    };

                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
