use anyhow::{Error, Result};
use flate2::{write::GzEncoder, Compression};
use std::{fs, fs::File, io::Write, net::TcpStream};

use super::{Request, Response};

pub fn handle_connection(mut stream: TcpStream, request: &str, directory: String) {
    let http_request = parse_request(request);
    let Request {
        method,
        route,
        user_agent,
        encoding,
        body,
    } = http_request.unwrap();

    let response = match method.as_str() {
        "GET" => match route.as_str() {
            "/" => Response::new().ok().content_length(0).build(),
            route if route.starts_with("/echo/") => {
                let content = route.replace("/echo/", "");

                let res = Response::new().ok().content_type("text/plain");
                if encoding.unwrap().contains("gzip") {
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(content.as_bytes()).unwrap();
                    let compressed = encoder.finish().unwrap();
                    res.content_encoding()
                        .content_length(compressed.len())
                        .body(compressed)
                        .build()
                } else {
                    res.content_length(content.len())
                        .body(content.into_bytes())
                        .build()
                }
            }
            route if route.starts_with("/user-agent") => {
                let agent = user_agent.unwrap();

                Response::new()
                    .ok()
                    .content_type("text/plain")
                    .content_length(agent.len())
                    .body(agent.into_bytes())
                    .build()
            }
            route if route.starts_with("/files/") => {
                let filename = route.replace("/files/", "");
                let mut filepath = directory;
                if !filepath.ends_with('/') {
                    filepath.push('/');
                }
                filepath.push_str(&filename);
                match fs::read(&filepath) {
                    Ok(content) => Response::new()
                        .ok()
                        .content_type("application/octet-stream")
                        .content_length(content.len())
                        .body(content)
                        .build(),
                    Err(e) => {
                        eprintln!("Failed to read the file: {e}");
                        Response::new().not_found().content_length(0).build()
                    }
                }
            }
            _ => Response::new().not_found().content_length(0).build(),
        },
        "POST" => {
            let filename = route.strip_prefix("/files/").unwrap();
            let mut filepath = directory;
            if !filepath.ends_with('/') {
                filepath.push('/');
            }
            filepath.push_str(filename);
            let mut file = match File::create(&filepath) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create file: {e}");
                    return;
                }
            };

            if let Err(e) = file.write_all(body.as_bytes()) {
                eprintln!("Failed to write to file: {e}");
            }

            Response::new().created().content_length(0).build()
        }
        _ => b"Invalid request method".to_vec(),
    };

    if let Err(e) = stream.write_all(&response) {
        println!("Failed to send response: {}", e);
    }
}

pub fn parse_request(request: &str) -> Result<Request, Error> {
    let (method_line, headers) = request.split_once("\r\n").unwrap();
    let (_, body) = headers.split_once("\r\n\r\n").unwrap();

    let method_line: Vec<&str> = method_line.split_whitespace().collect();
    let result = parse_header(headers);

    Ok(Request {
        method: String::from(method_line[0]),
        route: String::from(method_line[1]),
        user_agent: result.0,
        encoding: result.1,
        body: String::from(body),
    })
}

pub fn parse_header(headers: &str) -> (Option<String>, Option<String>) {
    let lines = headers.lines().collect::<Vec<&str>>();
    let user_agent = lines
        .iter()
        .find(|line| line.starts_with("User-Agent: "))
        .unwrap_or(&"")
        .replace("User-Agent: ", "");
    let encoding = lines
        .iter()
        .find(|line| line.starts_with("Accept-Encoding: "))
        .unwrap_or(&"")
        .replace("Accept-Encoding: ", "");

    (Some(user_agent), Some(encoding))
}
