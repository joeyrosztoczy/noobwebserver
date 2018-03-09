use std::net::TcpListener;

const PORT: i32 = 8080;

fn main() {
    let address = format!("127.0.0.1:{}", PORT);
    let listener = TcpListener::bind(address)
        .expect(&format!("Oops, port {} is already in use!", &PORT));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
    }
}
