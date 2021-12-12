use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
    println!();
    match TcpStream::connect("localhost:1234") {
        Ok(mut stream) => {
            println!("Connected to server on port 1234");

            let msg = b"##CN;";
            stream.write(msg).unwrap();
            println!("Sent CN, awaiting reply...");

            let mut data = [0 as u8; 1024];
            match stream.read(&mut data) {
                Ok(len) => {
                    println!("read {} bytes", len);
                    let text = from_utf8(&data).unwrap();
                    println!("Got reply: {}", text);
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
