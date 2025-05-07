# Rust Simple HTTP File Server

This project is a simple HTTP file server written in Rust. It serves files and directories from the project root over HTTP, allowing you to browse and download files using your web browser.

## Features
- Serves static files (HTML, images, PDFs, etc.) from the project directory
- Directory listing: browse folders and files via your browser
- Handles binary and text files
- Simple, single-threaded TCP server

## How It Works
- The server listens on `localhost:5500` by default.
- When you visit `http://localhost:5500/` in your browser, you see a list of files and directories in the project root.
- Clicking a file downloads or displays it (depending on your browser and file type).
- Clicking a directory navigates into it, showing its contents.

## Usage

### 1. Build and Run
Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```sh
cargo run
```

### 2. Open in Browser
Go to [http://localhost:5500/](http://localhost:5500/) in your web browser.

You should see a directory listing of your project. You can click on files to download/view them, or click into folders to browse further.

## Project Structure
- `src/main.rs`: Main server logic (socket, request handling)
- `src/http/request.rs`: HTTP request parsing
- `src/http/response.rs`: HTTP response and directory listing logic
- `index.html`: Example static file

## Notes
- The server only listens on `localhost` (127.0.0.1) for security.
- Large files are sent as binary data.
- The server is for local development and testing, not for production use.

## License
MIT
