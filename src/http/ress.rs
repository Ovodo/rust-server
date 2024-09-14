use infer;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::Path;
use url_escape::{encode_component_to_string,decode};

use super::request::HttpRequest;
use super::request::Version;

fn get_mime(path: &Path) -> String {
    // First, try to get MIME type using infer
    if let Some(kind) = infer::get_from_path(path).unwrap() {
        return kind.mime_type().to_string();
    }

    // Fallback to file extension
    let ext = path
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
    mime_map.insert("txt", "text/plain");
    mime_map.insert("toml", "text/plain");
    mime_map.insert("lock", "text/plain");

    mime_map
        .get(ext)
        .unwrap_or(&"application/octet-stream")
        .to_string()
}

#[derive(Debug)]
pub struct HttpResponse {
    version: Version,
    status: ResponseStatus,
    content_length: usize,
    accept_ranges: AcceptRanges,

    content_type: String, // Add content_type field
    pub response_body: String,
    pub current_path: String,
    pub binary_content: Option<Vec<u8>>, // Add binary_content field for non-text content
}

impl HttpResponse {
    pub fn new(request: &HttpRequest) -> io::Result<HttpResponse> {
        let version: Version = Version::V2_0;
        let mut status: ResponseStatus = ResponseStatus::NotFound;
        let mut content_length: usize = 0;
        let mut content_type = String::new();
        let mut response_body = String::new();
        let mut accept_ranges: AcceptRanges = AcceptRanges::None;
        let mut binary_content = None;

         // Decode the request path to handle any encoded characters
        let decoded_path = decode(&request.resource.path).into_owned();
        let current_path = decoded_path.clone();

        let server_path = std::env::current_dir()?;
        let new_path = server_path.join(&decoded_path);

        if prevent_backtracking(&new_path)? {
            if new_path.exists() {
                if new_path.is_file() {
                    accept_ranges = AcceptRanges::Bytes;
                    let content = fs::read(&new_path)?;
                    content_length = content.len();
                    status = ResponseStatus::OK;
                    accept_ranges = AcceptRanges::Bytes;
                    content_type = get_mime(&new_path);
                    // Check if content is text or binary
                    if content_type.starts_with("text/") {
                        // let content = format!(
                        //     "{} {}\n{}\ncontent-length: {}\r\n\r\n{}",
                        //     version, status, accept_ranges, content_length,&String::from_utf8_lossy(&content),
                        // );
                        response_body.push_str(&String::from_utf8_lossy(&content))
                    } else {
                        binary_content = Some(content); // For binary files
                      

                    }
                } else if new_path.is_dir() {
                    response_body.push_str("<html><body><h1>Directory Listing</h1><ul>");

                    // Add the "up" link to go up one directory
                    if let Some(parent_path) = Path::new(&decoded_path).parent() {
                        let parent_path_str = parent_path.to_str().unwrap_or("/");
                        response_body.push_str(&format!(
                            r#"<li><a href="/{}">up</a></li>"#,
                            parent_path_str.trim_start_matches("/")
                        ));
                    }

                    let dir_list = fs::read_dir(&new_path)?;
                    for entry in dir_list {
                        let entry = entry?;
                        let file_name = entry.file_name().into_string().unwrap_or_default();
                        let mut new = String::new();
                        let name = encode_component_to_string(file_name.clone(), &mut new );
                        let file_path = format!("{}/{}", decoded_path, name.to_string());
                        let display_name = if entry.path().is_dir() {
                            format!("{}/", file_name)
                        } else {
                            file_name
                        };

                        response_body.push_str(&format!(
                            r#"<li><a href="/{}">{}</a></li>"#,
                            file_path.trim_start_matches('/'), // Remove leading slash to avoid //
                            display_name
                        ));
                    }

                    response_body.push_str("</ul></body></html>");
                    content_length = response_body.len();
                    status = ResponseStatus::OK;
                    content_type = "text/html".to_string();
                }
            } else {
                // 404 Not Found
                status = ResponseStatus::NotFound;
                let not_found_page = "<html><body><h1>404 NOT FOUND</h1></body></html>";
                content_length = not_found_page.len();
                response_body.push_str(not_found_page);
                content_type = "text/html".to_string();
            }
        } else {
            // 403 Forbidden
            status = ResponseStatus::Forbidden;
            let forbidden_page = "<html><body><h1>403 Forbidden</h1></body></html>";
            content_length = forbidden_page.len();
            response_body.push_str(forbidden_page);
            content_type = "text/html".to_string();
        }

        let response = format!(
            "{} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            version, status, content_type, content_length, response_body
        );

        Ok(HttpResponse {
            version,
            status,
            content_length,
            content_type,
            accept_ranges,
            response_body: response,
            current_path,
            binary_content,
        })
    }
}

// Helper functions
fn prevent_backtracking(requested_path: &std::path::Path) -> Result<bool, io::Error> {
    // Ensure the path doesn't contain any `..` components before canonicalization
    for component in requested_path.components() {
        if let std::path::Component::ParentDir = component {
            println!("Backtracking detected in path: {:?}", requested_path);
            return Ok(false); // Block the request if backtracking is detected
        }
    }

    // Now, canonicalize the path to ensure it's within the allowed directory structure
    let root_cwd = std::env::current_dir()?;  // Root working directory
    let root_cwd_len = root_cwd.canonicalize()?.components().count();  // Count root components

    let resource_len = requested_path.canonicalize()?.components().count();  // Count requested components

    println!("Root path length: {}", root_cwd_len);
    println!("Requested path length: {}", resource_len);

    if root_cwd_len <= resource_len {
        println!("Backtracking prevented: allowed path");
        Ok(true)  // Path is allowed
    } else {
        println!("Backtracking detected: forbidden path");
        Ok(false)  // Backtracking detected
    }
}


#[derive(Debug)]
enum ResponseStatus {
    OK = 200,
    NotFound = 404,
    Forbidden = 403,
}

impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ResponseStatus::OK => "200 OK",
            ResponseStatus::NotFound => "404 NOT FOUND",
            ResponseStatus::Forbidden => "403 FORBIDDEN",
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
