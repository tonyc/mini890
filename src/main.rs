use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
// use std::vec;

const RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
const RESP_CONNECTION_ALLOWED: &str = "##CN1";
const CMD_REQUEST_CONNECTION: &str = "##CN;";

//const CMD_USER_PASS: &str = "##ID10705kenwoodadmin;";
const HOST: &str = "192.168.1.229:60000";
const USER: &str = "testuser";
const PASS: &str  = "testpass123!";
const BUFFER_SIZE:usize = 1286; // this appears to be the maximum size of the ##DD2 bandscope message

fn main() {
    let stream: TcpStream = TcpStream::connect(HOST).expect("Could not connect to server");

    println!("Connected to server on port 1234");

    match authenticate(&stream) {
        Ok(_stream) => {
            println!("Authentication successful!")
        }

        Err(msg) => {
            println!("Error authenticating: {}", msg)
        }
    }

    println!("Client Terminated.");
}

fn authenticate(mut stream: &TcpStream) -> Result<&TcpStream, &str> {
    let cmd_user_pass: String = make_login(&USER, &PASS);

    stream.write(CMD_REQUEST_CONNECTION.as_bytes()).unwrap();
    println!("Sent connection request CN, awaiting reply...");

    let mut buf = [0 as u8; BUFFER_SIZE];
    // let data: Vec<u8> = vec![];

    stream.read(&mut buf).unwrap();

    let text = from_utf8(&buf).unwrap();

    // BUG: text has the entire 1k buffer padded with zeroes
    println!("Read text: {}", text);

    let res = match find_cmd(text) {
        RESP_CONNECTION_ALLOWED => {
            println!("Sending username/password");
            stream.write(cmd_user_pass.as_bytes()).unwrap();

            // BUG: We should probably reset the data buffer each time we read
            stream.read(&mut buf).unwrap();

            let text = from_utf8(&buf).unwrap();
            println!("Authentication response: {}", text);

            let response = find_cmd(text);

            if RESP_AUTHENTICATION_SUCCESSFUL.eq(response) {
                Ok(stream)
            } else {
                Err("Incorrect username/password")
            }

        }

        _ => {
            Err("Connection denied")
        }
    };
    res
}

// slices a str up to the first semicolon
fn find_cmd(s: &str) -> &str {
    let pos: usize = s.find(";").unwrap();
    &s[..pos]
}

fn make_login(user: &str, pass: &str) -> String {
    let user_len = format!("{:0>2}", user.len());
    let pass_len = format!("{:0>2}", pass.len());

    format!("##ID1{}{}{}{};", user_len, pass_len, user, pass)
}

