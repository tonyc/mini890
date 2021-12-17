use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
// use std::vec;

fn main() {
    const RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
    const RESP_CONNECTION_ALLOWED: &str = "##CN1";

    const CMD_REQUEST_CONNECTION: &str = "##CN;";
    const CMD_USER_PASS: &str = "##ID10705kenwoodadmin;";

    const BUFFER_SIZE:usize = 1286;

    let mut stream: TcpStream = TcpStream::connect("localhost:1234").expect("Could not connect to server");

    println!("Connected to server on port 1234");

    stream.write(CMD_REQUEST_CONNECTION.as_bytes()).unwrap();
    println!("Sent connection request CN, awaiting reply...");

    let mut buf = [0 as u8; BUFFER_SIZE];
    // let data: Vec<u8> = vec![];

    stream.read(&mut buf).unwrap();

    let text = from_utf8(&buf).unwrap();

    // BUG: text has the entire 1k buffer padded with zeroes
    println!("Read text: {}", text);

    match find_cmd(text) {
        RESP_CONNECTION_ALLOWED => {
            println!("Sending username/password");
            stream.write(CMD_USER_PASS.as_bytes()).unwrap();

            // BUG: We should probably reset the data buffer each time we read
            stream.read(&mut buf).unwrap();

            let text = from_utf8(&buf).unwrap();
            println!("Authentication response: {}", text);

            let response = find_cmd(text);

            if RESP_AUTHENTICATION_SUCCESSFUL.eq(response) {
                println!("Successfully authenticated!");
            } else {
                println!("Incorrect username/password");
            }

        }

        _ => { println!("Connection denied"); }
    }

    println!("Client Terminated.");
}

// slices a str up to the first semicolon
fn find_cmd(s: &str) -> &str {
    let pos: usize = s.find(";").unwrap();
    &s[..pos]
}

