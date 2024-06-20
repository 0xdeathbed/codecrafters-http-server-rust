use anyhow::Result;
use bytes::Bytes;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    path::Path,
};
use tokio::{
    fs::{remove_file, OpenOptions},
    io::AsyncWriteExt,
    process::Command,
};

pub enum HttpStatus {
    Ok,
    NotFound,
    NotImplemented,
    Created,
    InternalServerError,
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            HttpStatus::Ok => "200 OK".to_string(),
            HttpStatus::NotFound => "404 Not Found".to_string(),
            HttpStatus::NotImplemented => "501 Not Implemented".to_string(),
            HttpStatus::Created => "201 Created".to_string(),
            HttpStatus::InternalServerError => "500 Internal Server Error".to_string(),
        };
        write!(f, "{}", value)
    }
}

pub struct HttpResponseBuilder {
    status_line: HttpStatus,
    headers: HashMap<String, String>,
    body: String,
    response: String,
    compression_detail: (bool, String),
    supported_compression: HashSet<&'static str>,
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_line: HttpStatus::Ok,
            response: String::new(),
            headers: HashMap::new(),
            body: String::new(),
            compression_detail: (false, String::new()),
            supported_compression: HashSet::from(["gzip"]),
        }
    }

    pub fn add_body_with_req_headers(&mut self, body: &str, media_type: &str) {
        self.body = body.to_string();

        self.add_header("Content-Type", media_type);
        self.add_header("Content-Length", &format!("{}", self.body.len()));
    }

    pub fn enable_compression(&mut self, compression_scheme: &str) {
        if let Some(scheme) = compression_scheme
            .split(',')
            .map(|v| v.trim())
            .find(|v| self.supported_compression.contains(v))
        {
            self.compression_detail.0 = true;
            self.compression_detail.1 = scheme.to_string();
            self.add_header("Content-Encoding", scheme);
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

    pub async fn compress(&mut self) -> Result<()> {
        let temp_file_path = "/tmp/tmp_compress";
        let mut temp = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(temp_file_path)
            .await?;
        temp.write_all(self.body.as_bytes()).await?;

        if self.compression_detail.1.as_str() == "gzip" {
            let gzip_output = Command::new("gzip")
                .args(["-c", "-n", temp_file_path])
                .output()
                .await?;

            self.body = unsafe { String::from_utf8_unchecked(gzip_output.stdout) }
        }

        remove_file(Path::new(temp_file_path)).await?;

        self.headers
            .insert("Content-Length".to_string(), format!("{}", self.body.len()));

        Ok(())
    }

    pub async fn build(&mut self) -> Result<Bytes> {
        if self.compression_detail.0 {
            self.compress().await?;
        }

        self.response = format!(
            "HTTP/1.1 {}\r\n{}\r\n{}",
            self.status_line,
            self.res_headers(),
            self.body
        );

        Ok(Bytes::from(self.response.clone()))
    }
}
