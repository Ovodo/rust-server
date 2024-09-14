use std::{
    io::{Read, Result, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

use simple_http::http::request;

fn create_socket() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5500)
}

fn handle_client(stream: &mut TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let buf_str = String::from_utf8_lossy(&buffer);
    let request = request::HttpRequest::new(&buf_str)?;
    let response = request.response()?;
    // println!("{:?}", &response);
    // println!("{}", &response.response_body);
    println!("{}", &response.current_path);
    // let body = response.response_body.clone();
    // Send binary content if present, else send response body as text
    if let Some(binary_content) = &response.binary_content {
        stream.write(response.response_body.as_bytes())?; // Send text response
        stream.write(binary_content)?; // Send binary content as bytes
    } else {
        stream.write(response.response_body.as_bytes())?; // Send text response
    }
    stream.flush()?;
    Ok(())
}
fn serve(socket: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(socket)?;
    let mut counter = 0;
    for stream in listener.incoming() {
        match std::thread::spawn(|| handle_client(&mut stream?)).join() {
            Ok(_) => {
                counter += 1;
                // println!("Connected stream... {}", counter)
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let socket = create_socket();
    serve(socket)?;
    Ok(())
}
