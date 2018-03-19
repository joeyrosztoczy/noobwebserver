extern crate noobwebserver;

use noobwebserver::ThreadPool;
// io::prelude gives us utilities to write to and from stream
use std::io::prelude::*;
// TcpListener binds a port to listen to incoming TcpStreams (i.e. if the browser tries to connect)
use std::net::TcpListener;
// TcpStream is a struct representing a TcpStream that can be read from and written too
use std::net::TcpStream;
use std::fs::File;
use std::time::Duration;
use std::thread;

// Convenience port (80x2 for http)
const PORT: i32 = 8080;

fn main() {
    let address = format!("127.0.0.1:{}", PORT);
    // This listener creats a new instance of a TcpListener struct, bound to the address & port
    // specified, clients can now attempt to connect to the server through this port
    let listener =
        TcpListener::bind(address).expect(&format!("Oops, port {} is already in use!", &PORT));

    let pool = ThreadPool::new(4);

    // Can ue the TcpListener accept method or iterate on incoming data with .incoming()
    for stream in listener.incoming() {
        println!("Connection established!");
        match stream {
            Ok(x) => {
                pool.async(|| { handle_connection(x) });
            },
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

    let get = b"GET / HTTP/1.1\r\n"; // b"" creates a byte string
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    // If the buffer starts with a get bethod, return our hello world
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        // sleep sync server to show shortcomings of single thread / synchronous
        thread::sleep(Duration::from_secs(5)); 
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    // Open up our html file for the given http method
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    // Follow HTTP response protocol HTTP/version status-code reason-txt no header no body
    let response = format!("{}{}", status_line, contents);
    // Convert response to a utf8 slice and write to the stream, write returns a Result, so we
    // unwrap
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
