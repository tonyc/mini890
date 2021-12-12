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

            stream.read(&mut data).unwrap();
            let text = from_utf8(&data).unwrap();

            // BUG: text has the entire 1k buffer padded with zeroes
            println!("Read text: {}", text);

            let response = read_cmd(text);

            if RESP_CONNECTION_ALLOWED.eq(response) {
                println!("Sending username/password");
                stream.write(CMD_USER_PASS.as_bytes()).unwrap();

                // BUG: We should probably reset the data buffer each time we read
                stream.read(&mut data).unwrap();

                let text = from_utf8(&data).unwrap();
                println!("Authentication response: {}", text);

                let response = read_cmd(text);

                if RESP_AUTHENTICATION_SUCCESSFUL.eq(response) {
                    println!("Successfully authenticated!");
                } else {
                    println!("Incorrect username/password");
                }
            } else {
                println!("Connection denied");
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Client Terminated.");
}

// slices a str up to the first semicolon
fn read_cmd(s: &str) -> &str {
    let pos = s.find(";").unwrap();
    &s[..pos]
}

