use super::*;
use std::{borrow::Cow, fs};

pub fn echo(request: &Cow<str>, path: &str) -> String {
    let mut encoding = "";
    for header in request.lines() {
        if header.starts_with("Accept-Encoding:") {
            encoding = header.trim_start_matches("Accept-Encoding:").trim();
            break;
        }
    }

    let response = path.trim_start_matches("/echo/");

    format!(
        "{}Content-Type: text/plain{}\r\nContent-Length: {}\r\n\r\n{}",
        OK.part(),
        if encoding.contains("gzip") {
            "\r\nContent-Encoding: gzip"
        } else {
            ""
        },
        response.len(),
        response
    )
}

pub fn user_agent(path: &Cow<str>) -> String {
    let mut agent = "";
    for header in path.lines() {
        if header.starts_with("User-Agent:") {
            agent = header.trim_start_matches("User-Agent:").trim();
            break;
        }
    }
    format!(
        "{}Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        OK.part(),
        agent.len(),
        agent
    )
}

pub fn file(path: &str) -> String {
    let filename = &path[7..];
    let path_to_read = format!("/tmp/data/codecrafters.io/http-server-tester/{}", filename);

    let response = match fs::read(path_to_read) {
        Ok(contents) => format!(
            "{}Content-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
            OK.part(),
            contents.len(),
            String::from_utf8_lossy(&contents)
        ),
        Err(_) => NotFound.whole(),
    };

    println!("Response: {}", response);

    response
}

pub fn create_file(request: &Cow<str>, path: &str) -> String {
    let filename = &path[7..];
    let req_body = request.lines().last().unwrap();
    let path_to_write = format!("/tmp/data/codecrafters.io/http-server-tester/{}", filename);

    if let Ok(_) = fs::write(&path_to_write, req_body) {
        Created.whole()
    } else {
        InternalServer.whole()
    }
}
