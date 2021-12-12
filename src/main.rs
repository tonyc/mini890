use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
    let connection_allowed = String::from("##CN1");

    match TcpStream::connect("localhost:1234") {
        Ok(mut stream) => {
            println!("Connected to server on port 1234");

            let msg = b"##CN;";
            stream.write(msg).unwrap();
            println!("Sent connection request CN, awaiting reply...");

            let mut data = [0 as u8; 1024];

            match stream.read(&mut data) {
                Ok(len) => {
                    println!("read {} bytes", len);

                    let text = from_utf8(&data).unwrap();
                    println!("Read text: {}", text);

                    let pos = text.find(";").unwrap();
                    println!("Found separator at position: {}", pos);

                    //let text_len = text.len();
                    //println!("Got reply: {:?} with len: {}", text, text_len);
                    //println!("sliced text: {}", text[..pos]);
                    //let sliced = &text[0..pos];
                    //println!("sliced: {}", sliced);

                    if connection_allowed.eq(&text[0..pos]) {
                        println!("Sending username/password");
                        stream.write(b"##ID10705kenwoodadmin;").unwrap();

                        match stream.read(&mut data) {
                            Ok(_) => {
                                let text = from_utf8(&data).unwrap();
                                println!("Reply from l/p: {}", text);

                                if String::from("##ID1;").eq(text) {
                                    println!("Successfully authenticated!");
                                } else {
                                    println!("Incorrect username/password");
                                }
                            },
                            Err(e) => {
                                println!("Error receiving data: {}", e);
                            }
                        }

                    } else {
                        println!("Connection denied");
                    }

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
    println!("Client Terminated.");
}

