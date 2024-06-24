// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);

    let mut headers = request.lines();
    if let Some(request_line) = headers.next() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() == 3 {
            let request_header = parts[0] == "GET";
            let path = parts[1];

            let mut user_agent = "unknown";

            for header in headers {
                if header.starts_with("User-Agent") {
                    user_agent = header.trim_start_matches("User-Agent:").trim();
                    break;
                }
            }

            let response = if request_header && path == "/user-agent" {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    user_agent.len(),
                    user_agent
                )
            } else if request_header && path == "/" {
                format!("HTTP/1.1 200 OK\r\n\r\n")
            } else if request_header && path.starts_with("/echo/") {
                let response_body = path.trim_start_matches("/echo/");
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    response_body.len(),
                    response_body
                )
            } else {
                format!("HTTP/1.1 404 Not Found\r\n\r\n")
            };
            stream.write(response.as_bytes()).unwrap();
        }
    }
}
