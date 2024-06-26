pub enum StatusCode {
    OK = 200,
    NotFound = 404,
    Created = 201,
}

impl StatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OK => "200 OK",
            Self::Created => "201 Created",
            Self::NotFound => "404 Not Found",
        }
    }

    pub fn part(&self) -> String {
        format!("HTTP/1.1 {}\r\n", self.as_str())
    }

    pub fn whole(&self) -> String {
        format!("HTTP/1.1 {}\r\n\r\n", self.as_str())
    }
}

#[derive(Debug)]
pub enum Request {
    GET,
    POST,
    NONE,
}

pub fn extract_request_method_and_path(header_line: Option<&str>) -> (Request, &str) {
    let request: Vec<&str> = header_line.unwrap().split_whitespace().collect();
    let method = request[0];
    let path = request[1];
    use Request::*;
    let val = match method {
        "GET" => GET,
        "POST" => POST,
        _ => NONE,
    };
    (val, path)
}
