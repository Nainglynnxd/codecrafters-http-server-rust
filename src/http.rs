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

    pub fn _compose_response(&self, body: &str) -> String {
        format!("HTTP/1.1 {}\r\n{}", self.as_str(), body)
    }

    pub fn default(&self) -> String {
        format!("HTTP/1.1 {}\r\n\r\n", self.as_str())
    }
}

pub enum Request {
    GET,
    POST,
    NONE,
}
