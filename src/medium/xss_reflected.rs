use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use serde::{Deserialize, Serialize};

pub fn run() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to address");
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1028];

    match stream.read(&mut buffer) {
        Ok(size) => {
            println!("Received {} bytes", size);
            let request = String::from_utf8_lossy(&buffer[0..size]);
            println!("{}", request);
            println!("handled request");

            // Check if this is an OPTIONS request (CORS preflight)
            if request.starts_with("OPTIONS") {
                let cors_response = "HTTP/1.1 200 OK\r\n\
                    Access-Control-Allow-Origin: http://10.164.2.70\r\n\
                    Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                    Access-Control-Allow-Headers: access-control-allow-origin, content-type\r\n\
                    Access-Control-Max-Age: 86400\r\n\
                    Content-Length: 0\r\n\
                    Connection: close\r\n\r\n";

                stream.write(cors_response.as_bytes()).unwrap();
                println!("Sent CORS preflight response");
            } else {
                // For non-OPTIONS requests (GET, POST, etc.)
                let response = "HTTP/1.1 200 OK\r\n\
                    Access-Control-Allow-Origin: http://10.164.2.70\r\n\
                    Content-Type: text/plain\r\n\
                    Connection: close\r\n\
                    Content-Length: 25\r\n\r\n\
                    Data received successfully!";

                stream.write(response.as_bytes()).unwrap();
                println!("Sent standard response");
            }

            stream.flush().unwrap();
            stream
                .shutdown(std::net::Shutdown::Both)
                .unwrap_or_else(|e| {
                    eprintln!("Error shutting down connection: {}", e);
                });
        }
        Err(e) => {
            eprintln!("Error reading from connection: {}", e);
        }
    }
}
