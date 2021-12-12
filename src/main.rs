use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;

fn main() {
    static RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
    static RESP_CONNECTION_ALLOWED: &str = "##CN1";

    static CMD_REQUEST_CONNECTION: &str = "##CN;";
    static CMD_USER_PASS: &str = "##ID10705kenwoodadmin;";

    match TcpStream::connect("localhost:1234") {
        Ok(mut stream) => {
            println!("Connected to server on port 1234");

            stream.write(CMD_REQUEST_CONNECTION.as_bytes()).unwrap();
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

                    if RESP_CONNECTION_ALLOWED.eq(&text[0..pos]) {
                        println!("Sending username/password");
                        stream.write(CMD_USER_PASS.as_bytes()).unwrap();

                        match stream.read(&mut data) {
                            Ok(_) => {
                                let text = from_utf8(&data).unwrap();
                                println!("Reply from l/p: {}", text);

                                let pos = text.find(";").unwrap();
                                let response = &text[0..pos];
                                println!("response: {}", response);

                                if RESP_AUTHENTICATION_SUCCESSFUL.eq(response) {
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
