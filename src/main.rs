// io::prelude gives us utilities to write to and from stream
use std::io::prelude::*;
// TcpListener binds a port to listen to incoming TcpStreams (i.e. if the browser tries to connect)
use std::net::TcpListener;
// TcpStream is a struct representing a TcpStream that can be read from and written too
use std::net::TcpStream;
use std::fs::File;

// Convenience port (80x2 for http)
const PORT: i32 = 8080;

fn main() {
    let address = format!("127.0.0.1:{}", PORT);
    // This listener creats a new instance of a TcpListener struct, bound to the address & port
    // specified, clients can now attempt to connect to the server through this port
    let listener =
        TcpListener::bind(address).expect(&format!("Oops, port {} is already in use!", &PORT));

    // Can ue the TcpListener accept method or iterate on incoming data with .incoming()
    for stream in listener.incoming() {
        println!("Connection established!");
        match stream {
            Ok(x) => handle_connection(x),
            Err(e) => panic!("Shit something went wrong {:?}", e),
        };
    }
}
// handle_connection takes a stream as input (must be mut bc read mutates the struct
fn handle_connection(mut stream: TcpStream) {
    // Create a buffer to read the stream into
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    println!("Stream: {}", String::from_utf8_lossy(&buffer[..]));

    // Open up our html file
    let mut file = File::open("hello.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    // Follow HTTP response protocol HTTP/version status-code reason-txt no header no body
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
    // Convert response to a utf8 slice and write to the stream, write returns a Result, so we
    // unwrap
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
