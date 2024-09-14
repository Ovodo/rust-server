use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::Path;

use super::request::HttpRequest;
use super::request::Version;

fn get_mime_type(content: &[u8], file_name: &str) -> String {
    // First, try to get MIME type using infer
    if let Some(kind) = infer::get(content) {
        return kind.mime_type().to_string();
    }

    // Fallback to file extension
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let mut mime_map = HashMap::new();
    mime_map.insert("html", "text/html");
    mime_map.insert("css", "text/css");
    mime_map.insert("js", "application/javascript");
    mime_map.insert("json", "application/json");
    mime_map.insert("png", "image/png");
    mime_map.insert("jpg", "image/jpeg");
    mime_map.insert("jpeg", "image/jpeg");
    mime_map.insert("gif", "image/gif");
    mime_map.insert("svg", "image/svg+xml");
    mime_map.insert("pdf", "application/pdf");
    mime_map.insert("mp4", "video/mp4");

    mime_map
        .get(ext)
        .unwrap_or(&"application/octet-stream")
        .to_string()
}

fn get_mime(path: &Path) -> String {
    if let Some(kind) = infer::get_from_path(path).unwrap() {
        kind.mime_type().to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    version: Version,
    status: ResponseStatus,
    content_length: usize,
    accept_ranges: AcceptRanges,
    pub response_body: String,
    pub current_path: String,
}

impl HttpResponse {
    pub fn new(request: &HttpRequest) -> io::Result<HttpResponse> {
        let version: Version = Version::V2_0;
        let mut status: ResponseStatus = ResponseStatus::NotFound;
        let mut content_length: usize = 0;
        let mut content_type = String::new();
        let mut accept_ranges: AcceptRanges = AcceptRanges::None;
        let current_path = request.resource.path.clone();
        let mut response_body = String::new();

        let resource = &request.resource.path;
        let server_path = std::env::current_dir()?;
        let new_path = server_path.join(resource);
        if new_path.exists() {
            if new_path.is_file() {
                let content = std::fs::read_to_string(&new_path)?;
                content_length = content.len();
                status = ResponseStatus::OK;
                accept_ranges = AcceptRanges::Bytes;
                let content_type = get_mime(&new_path);
                println!("content_type: {}", content_type);
                let content = format!(
                    "{} {}\n{}\ncontent-length: {}\r\n\r\n{}",
                    version, status, accept_ranges, content_length, content
                );
                response_body.push_str(&content)
            } else if new_path.is_dir() {
                let mut body = String::new();
                body.push_str("<html><body><h1>Directory Listing</h1><ul>");

                // Add the "up" link to go up one directory
                if let Some(parent_path) = new_path.parent() {
                    let parent_path_str = parent_path.to_str().unwrap_or("/");
                    body.push_str(&format!(
                        r#"<li><a href="/{}">Up</a></li>"#,
                        parent_path_str.trim_start_matches("/")
                    ));
                }

                let dir_list = fs::read_dir(&new_path)?;

                for entry in dir_list {
                    let entry = entry?;
                    let file_name = entry.file_name().into_string().unwrap_or_default();
                    let file_path = format!("{}/{}", resource, file_name);

                    let display_name = if entry.path().is_dir() {
                        format!("{}/", file_name)
                    } else {
                        file_name
                    };

                    body.push_str(&format!(
                        r#"<li><a href="/{}">{}</a></li>"#,
                        file_path.trim_start_matches('/'), // Remove leading slash to avoid //
                        display_name
                    ));
                }

                body.push_str("</ul></body></html>");
                content_length = body.len();
                status = ResponseStatus::OK;
                content_type = "text/html".to_string();
                let content = format!(
                    "{} {}\n{}\ncontent-length: {}  \r\n\r\n{}
              
              ",
                    version, status, accept_ranges, content_length, body
                );
                response_body.push_str(&content)
            } else {
                let four_o_four = "<html>
                <body>
                <h1>
                404 NOT FOUND
                </h1>
                </body>
                </html>";
                content_length = four_o_four.len();
                let content = format!(
                    "{} {}\n{}\ncontent-length: {}  \r\n\r\n{}
              
              ",
                    version, status, accept_ranges, content_length, four_o_four
                );
                response_body.push_str(&content)
            }
        }
        Ok(HttpResponse {
            version,
            status,
            content_length,
            accept_ranges,
            response_body,
            current_path,
        })
    }
}
#[derive(Debug)]
enum ResponseStatus {
    OK = 200,
    NotFound = 404,
}

impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ResponseStatus::OK => "200 OK",
            ResponseStatus::NotFound => "404 NOT FOUND",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]

enum AcceptRanges {
    Bytes,
    None,
}

impl Display for AcceptRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AcceptRanges::Bytes => "accept-ranges: bytes",
            AcceptRanges::None => "accept-ranges: none",
        };
        write!(f, "{}", msg)
    }
}
