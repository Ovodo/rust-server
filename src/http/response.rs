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
                    // Start of beautiful HTML directory listing (HTML and style only)
                    response_body.push_str(r#"
<!DOCTYPE html>
<html lang='en'>
<head>
  <meta charset='UTF-8'>
  <title>Ovdizzle Directory Listing</title>
  <link href='https://fonts.googleapis.com/css?family=Segoe+UI:400,700&display=swap' rel='stylesheet'>
  <style>
    body {
      margin: 0; padding: 0;
      min-height: 100vh;
      background: radial-gradient(circle at 60% 40%, #ffe066 0%, #ff9966 40%, #ff5e62 100%);
      font-family: 'Segoe UI', Arial, sans-serif;
      display: flex; flex-direction: column; align-items: center;
    }
    .container {
      background: rgba(255,255,255,0.18);
      box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.18);
      backdrop-filter: blur(8px);
      border-radius: 28px;
      padding: 48px 36px 36px 36px;
      margin-top: 60px;
      min-width: 340px;
      max-width: 600px;
      width: 90vw;
      border: 1.5px solid rgba(255,255,255,0.22);
    }
    h1 {
      color: #fff;
      font-size: 2.3em;
      margin-bottom: 20px;
      letter-spacing: 2px;
      text-shadow: 2px 2px 12px #ff5e62, 0 1px 0 #ffe066;
    }
    ul {
      list-style: none;
      padding: 0;
      margin: 0;
    }
    li {
      margin: 12px 0;
      text-align: left;
    }
    a {
      color: #ff5e62;
      background: #ffe066;
      padding: 10px 22px;
      border-radius: 10px;
      text-decoration: none;
      font-weight: bold;
      box-shadow: 0 2px 8px rgba(255,224,102,0.13);
      transition: background 0.2s, color 0.2s, box-shadow 0.2s;
      display: inline-block;
      font-size: 1.08em;
      border: 1.5px solid #ff5e62;
    }
    a:hover {
      background: #ff5e62;
      color: #ffe066;
      box-shadow: 0 4px 16px rgba(255,94,98,0.18);
    }
    .up-link {
      color: #fff;
      background: #ff5e62;
      font-size: 1em;
      margin-bottom: 16px;
      border: 1.5px solid #ffe066;
      box-shadow: 0 2px 8px rgba(255,94,98,0.13);
    }
    .up-link:hover {
      background: #ffe066;
      color: #ff5e62;
      border: 1.5px solid #ff5e62;
    }
    .dir:after {
      content: '  📁';
      margin-left: 8px;
    }
    .file:after {
      content: '  📄';
      margin-left: 8px;
    }
  </style>
</head>
<body>
  <div class='container'>
    <h1>Directory Listing</h1>
    <ul>
"#);

                    // Add the "up" link to go up one directory
                    if let Some(parent_path) = Path::new(&decoded_path).parent() {
                        let parent_path_str = parent_path.to_str().unwrap_or("");
                        // Avoid double slashes and encoding issues
                        let up_href = if parent_path_str.is_empty() { "" } else { parent_path_str.trim_start_matches('/') };
                        response_body.push_str(&format!(
                            r#"      <li><a class='up-link' href='/{}'>⬆️ Up</a></li>"#,
                            up_href
                        ));
                    }

                    let dir_list = fs::read_dir(&new_path)?;
                    for entry in dir_list {
                        let entry = entry?;
                        let file_name = entry.file_name().into_string().unwrap_or_default();
                        // Only encode the file name, not the whole path, and do not add quotes
                        let mut encoded = String::new();
                        let encoded_file_name = encode_component_to_string(&file_name, &mut encoded);
                        let file_path = if decoded_path.is_empty() {
                            encoded_file_name.to_string()
                        } else {
                            format!("{}/{}", decoded_path.trim_end_matches('/'), encoded_file_name)
                        };
                        let is_dir = entry.path().is_dir();
                        let display_name = if is_dir {
                            format!("{}/", file_name)
                        } else {
                            file_name.clone()
                        };
                        let class = if is_dir { "dir" } else { "file" };
                        // Only encode the file_path for the href, not the display_name
                        response_body.push_str(&format!(
                            r#"      <li><a class='{}' href='/{}'>{}</a></li>"#,
                            class,
                            file_path.trim_start_matches('/'),
                            display_name
                        ));
                    }

                    response_body.push_str(r#"    </ul>
  </div>
  <footer style=\"margin-top: 40px; color: #fff; font-size: 16px; opacity: 0.8;\">&copy; 2025 Ovdizzle Server</footer>
</body>
</html>
"#);
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
