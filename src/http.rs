use bytes::Bytes;
use std::collections::{HashMap, HashSet};

pub enum HttpStatus {
    Ok,
    NotFound,
    NotImplemented,
    Created,
    InternalServerError,
}

impl ToString for HttpStatus {
    fn to_string(&self) -> String {
        match self {
            HttpStatus::Ok => "200 OK".to_string(),
            HttpStatus::NotFound => "404 Not Found".to_string(),
            HttpStatus::NotImplemented => "501 Not Implemented".to_string(),
            HttpStatus::Created => "201 Created".to_string(),
            HttpStatus::InternalServerError => "500 Internal Server Error".to_string(),
        }
    }
}

pub struct HttpResponseBuilder {
    status_line: HttpStatus,
    headers: HashMap<String, String>,
    body: String,
    response: String,
    is_compression: bool,
    supported_compression: HashSet<&'static str>,
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_line: HttpStatus::Ok,
            response: String::new(),
            headers: HashMap::new(),
            body: String::new(),
            is_compression: false,
            supported_compression: HashSet::from(["gzip"]),
        }
    }

    pub fn add_body_with_req_headers(&mut self, body: &str, media_type: &str) {
        self.body = body.to_string();

        self.add_header("Content-Type", media_type);
        self.add_header("Content-Length", &format!("{}", self.body.len()));
    }

    pub fn enable_compression(&mut self, compression_scheme: &str) {
        match compression_scheme
            .split(",")
            .map(|v| v.trim())
            .find(|v| self.supported_compression.contains(v))
        {
            Some(scheme) => {
                self.is_compression = true;
                self.add_header("Content-Encoding", scheme);
            }
            None => {}
        }
    }

    pub fn add_status(&mut self, status: HttpStatus) {
        self.status_line = status;
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    fn res_headers(&self) -> String {
        let mut headers = String::new();
        for (k, v) in &self.headers {
            headers.push_str(&format!("{k}: {v}\r\n"));
        }
        headers
    }

    pub fn compress(&mut self) {
        self.headers
            .insert("Content-Length".to_string(), format!("{}", self.body.len()));
    }

    pub fn build(&mut self) -> Bytes {
        if self.is_compression {
            self.compress();
        }

        self.response = format!(
            "HTTP/1.1 {}\r\n{}\r\n{}",
            self.status_line.to_string(),
            self.res_headers(),
            self.body
        );

        Bytes::from(self.response.clone())
    }
}
