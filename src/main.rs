mod http;
mod utils;

use anyhow::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use http::HttpRequest;
use std::fmt::{self};
use std::net::{TcpListener, TcpStream};
use std::{fs, io::Write, thread};
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

fn handle_connection(mut stream: TcpStream, request: &str, directory: String) {
    let mut response = String::new();
    let http_request = parse_request(request);
    let Request {
        method,
        route,
        user_agent,
        encoding,
    } = http_request.unwrap();

    match method.as_str() {
        "GET" => match route.as_str() {
            "/" => {
                let res = Response {
                    status_code: 200,
                    ..Response::default()
                };
                response.push_str(&res.to_string());
            }
            route if route.starts_with("/echo/") => {
                let content = route.replace("/echo/", "");
                let compressed = if !encoding.as_ref().unwrap().is_empty() {
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(content.as_bytes()).unwrap();
                    let compressed_body = encoder.finish().unwrap();
                    unsafe { String::from_utf8_unchecked(compressed_body) }
                } else {
                    content
                };

                let res = Response {
                    status_code: 200,
                    content_type: String::from("text/plain"),
                    content_length: compressed.len() as i16,
                    body: compressed,
                    content_encoding: encoding.unwrap(),
                };
                response.push_str(&res.to_string());
            }
            route if route.starts_with("/user-agent") => {
                let res = Response {
                    status_code: 200,
                    content_type: String::from("text/plain"),
                    content_length: user_agent.as_ref().unwrap().len() as i16,
                    body: user_agent.unwrap(),
                    ..Response::default()
                };
                response.push_str(&res.to_string());
            }
            route if route.starts_with("/files/") => {
                let filename = route.replace("/files/", "");
                let mut filepath = directory;
                if !filepath.ends_with("/") {
                    filepath.push('/');
                }
                filepath.push_str(&filename);
            }
            _ => response.push_str(Response::NOT_FOUND),
        },
        "POST" => {}
        _ => {
            eprintln!("Invalid request method");
        }
    };

    let response = response.as_bytes();
    stream.write_all(response).unwrap();
}

fn parse_request(request: &str) -> Result<Request, Error> {
    let (method_line, headers) = request.split_once("\r\n").unwrap();

    let method_line: Vec<&str> = method_line.split_whitespace().collect();
    let (user_agent, encoding) = parse_header(headers);

    Ok(Request {
        method: String::from(method_line[0]),
        route: String::from(method_line[1]),
        user_agent,
        encoding,
    })
}

fn parse_header(headers: &str) -> (Option<String>, Option<String>) {
    let mut lines = headers.lines();
    let user_agent = lines
        .find(|line| line.starts_with("User-Agent: "))
        .unwrap_or("")
        .replace("User-Agent: ", "");
    let encoding = lines
        .find(|line| line.starts_with("Accept-Encoding: "))
        .unwrap_or("")
        .replace("Accept-Encoding: ", "");

    (Some(user_agent), Some(encoding))
}

#[derive(Debug)]
struct Request {
    method: String,
    route: String,
    user_agent: Option<String>,
    encoding: Option<String>,
}

#[derive(Default)]
struct Response {
    status_code: u16,
    content_type: String,
    content_encoding: String,
    content_length: i16,
    body: String,
}

// impl Default for Response {
//     fn default() -> Self {
//         Response {
//             status_code: Default::default(),
//             content_type: Default::default(),
//             content_encoding: Default::default(),
//             content_length: Default::default(),
//             body: Default::default(),
//         }
//     }
// }

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP/1.1 {} OK\r\n", self.status_code)?;

        if !self.content_type.is_empty() {
            write!(f, "Content-Type: {}\r\n", self.content_type)?;
        }

        if !self.content_encoding.is_empty() {
            write!(f, "Content-Encoding: {}\r\n", self.content_encoding)?;
        }

        if self.content_length >= 0 {
            write!(f, "Content-Length: {}\r\n", self.content_length)?;
        }

        write!(f, "\r\n")?;

        if !self.body.is_empty() {
            write!(f, "{}", self.body)?;
        }

        Ok(())
    }
}

impl Response {
    const NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found\r\n\r\n";
}
