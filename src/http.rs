use bytes::Bytes;
use std::collections::HashMap;

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
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_line: HttpStatus::Ok,
            response: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn add_body_with_req_headers(&mut self, body: &str, body_type: &str) {
        self.body = body.to_string();

        self.add_header("Content-Type".to_string(), body_type.to_string());
        self.add_header("Content-Length".to_string(), format!("{}", self.body.len()));
    }

    pub fn add_status(&mut self, status: HttpStatus) {
        self.status_line = status;
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    fn res_headers(&self) -> String {
        let mut headers = String::new();
        for (k, v) in &self.headers {
            headers.push_str(&format!("{k}: {v}\r\n"));
        }
        headers
    }

    pub fn build(&mut self) -> &Self {
        self.response = format!(
            "HTTP/1.1 {}\r\n{}\r\n{}",
            self.status_line.to_string(),
            self.res_headers(),
            self.body
        );

        self
    }

    pub fn convert_to_bytes(&self) -> Bytes {
        Bytes::from(self.response.clone())
    }
}
