#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub route: String,
    pub user_agent: Option<String>,
    pub encoding: Option<String>,
    pub body: String,
}

pub struct Response {
    pub status_line: Vec<u8>,
    pub headers: Vec<u8>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status_line: Vec::new(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn ok(mut self) -> Self {
        self.status_line = b"HTTP/1.1 200 OK\r\n".to_vec();
        self
    }

    pub fn created(mut self) -> Self {
        self.status_line = b"HTTP/1.1 201 Created\r\n".to_vec();
        self
    }

    pub fn not_found(mut self) -> Self {
        self.status_line = b"HTTP/1.1 404 Not Found\r\n".to_vec();
        self
    }

    pub fn content_encoding(mut self) -> Self {
        self.headers
            .extend_from_slice(b"Content-Encoding: gzip\r\n");
        self
    }

    pub fn content_type(mut self, content_type: &str) -> Self {
        self.headers
            .extend_from_slice(format!("Content-Type: {}\r\n", content_type).as_bytes());
        self
    }

    pub fn content_length(mut self, len: usize) -> Self {
        self.headers
            .extend_from_slice(format!("Content-Length: {}\r\n", len).as_bytes());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn build(self) -> Vec<u8> {
        let mut response = Vec::new();
        response.extend_from_slice(&self.status_line);
        response.extend_from_slice(&self.headers);
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);
        response
    }
}
