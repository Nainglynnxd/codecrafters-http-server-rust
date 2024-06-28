#![allow(dead_code)]
use super::*;

pub fn parse_request(request: &str) -> Result<HttpRequest, Error> {
    let lines = request.lines().collect::<Vec<&str>>();
    let mut req_header = lines[0].split_whitespace();
    let user_agent = parse_header("User-Agent: ", &lines);
    let encoding = parse_header("Accept-Encoding: ", &lines);
    let http_request = HttpRequest {
        path: String::from(req_header.nth(1).unwrap_or("")),
        user_agent: Some(user_agent),
        valid_encoding: Some(encoding),
    };
    Ok(http_request)
}

pub fn parse_header(header: &str, lines: &[&str]) -> String {
    lines
        .iter()
        .find(|line| line.starts_with(header))
        .unwrap_or(&"")
        .replace(header, "")
}

pub fn file_path_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            println!("File metadata for path {}: {:?}", path, metadata);
            metadata.is_file()
        }
        Err(e) => {
            println!("Failed to get metadata for path {}: {}", path, e);
            false
        }
    }
}
