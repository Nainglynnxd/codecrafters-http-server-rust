pub struct HttpRequest {
    pub path: String,
    pub user_agent: Option<String>,
    pub valid_encoding: Option<String>,
}

impl Default for HttpRequest {
    /// Default values for the Request
    fn default() -> Self {
        HttpRequest {
            path: Default::default(),
            user_agent: Default::default(),
            valid_encoding: Default::default(),
        }
    }
}
