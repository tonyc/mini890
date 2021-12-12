use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

fn main() {
    let request_connection = String::from("##CN;");
    let connection_allowed = String::from("##CN1");
    let authentication_successful = String::from("##ID1");

    match TcpStream::connect("localhost:1234") {
        Ok(mut stream) => {
            println!("Connected to server on port 1234");

            stream.write(request_connection.as_bytes()).unwrap();
            println!("Sent connection request CN, awaiting reply...");

            let mut data = [0 as u8; 1024];

            match stream.read(&mut data) {
                Ok(len) => {
                    println!("read {} bytes", len);

                    let text = from_utf8(&data).unwrap();

                    // bug: text has the entire 1k buffer padded with zeroes
                    println!("Read text: {}", text);

                    let pos = text.find(";").unwrap();
                    println!("Found separator at position: {}", pos);

                    if connection_allowed.eq(&text[0..pos]) {
                        println!("Sending username/password");
                        stream.write(b"##ID10705kenwoodadmin;").unwrap();

                        match stream.read(&mut data) {
                            Ok(_) => {
                                let text = from_utf8(&data).unwrap();
                                println!("Reply from l/p: {}", text);

                                let pos = text.find(";").unwrap();
                                let response = &text[0..pos];
                                println!("response: {}", response);

                                if authentication_successful.eq(&response) {
                                    println!("Successfully authenticated!");
                                } else {
                                    println!("Incorrect username/password");
                                }
                            }
                            Err(e) => {
                                println!("Error receiving data: {}", e);
                            }
                        }
                    } else {
                        println!("Connection denied");
                    }
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Client Terminated.");
}
