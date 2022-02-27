use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

const RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
const RESP_CONNECTION_ALLOWED: &str = "##CN1";
const CMD_REQUEST_CONNECTION: &str = "##CN;";

//const CMD_USER_PASS: &str = "##ID10705kenwoodadmin;";
const HOST: &str = "192.168.1.229:60000";
const USER: &str = "testuser";
const PASS: &str = "testpass123!";
const BUFFER_SIZE: usize = 1286; // this appears to be the maximum size of the ##DD2 bandscope message

fn main() {
    let mut stream: TcpStream = TcpStream::connect(HOST).expect("Could not connect to server");

    println!("Connected to server on port 1234");

    radio_authenticate(&stream).expect("Could not authenticate");
    println!("Authentication successful!");

    let (tx, _rx) = mpsc::channel();

    let timer_thread = thread::spawn(move || {
        println!("spawning timer thread");
        loop {
            println!(" --> async thread!");
            //send_cmd(&stream, "PS;");
            tx.send("PS").unwrap();
            sleep(1000);
        }
    });

    let connection_thread = thread::spawn(move || {
        println!("spawning connection thread");

        send_cmd(&stream, "AI2;").unwrap();
        //send_cmd(&stream, "DD11;").unwrap();

        let mut buf = [' ' as u8; BUFFER_SIZE];
        loop {
            send_cmd(&stream, "PS;").unwrap();

            match stream.read(&mut buf) {
                Ok(0) => {
                    println!("No bytes to read. Did the radio drop the connection?");
                    break;
                }

                Ok(n) => {
                    let text = from_utf8(&buf[0..n]).unwrap();
                    let mut cmds: Vec<&str> = vec![];

                    for cmd in text.split_terminator(";") {
                        cmds.push(cmd);
                    }

                    for cmd in cmds {
                        println!("[DN] {}", cmd);
                    }

                    // reset the buffer
                    buf.iter_mut().for_each(|x| *x = ' ' as u8);
                }

                Err(other) => {
                    println!("Error reading stream: {:?}", other);
                    break;
                }
            }

            sleep(500);
        } // loop

        println!("Client Terminated.");
    });

    timer_thread.join().unwrap();
    connection_thread.join().unwrap();
}

fn radio_authenticate(mut stream: &TcpStream) -> Result<&TcpStream, &str> {
    send_cmd(&stream, &CMD_REQUEST_CONNECTION).unwrap();
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
            send_cmd(&stream, &login_cmd(USER, PASS)).unwrap();

            // BUG: We should probably reset the data buffer each time we read
            stream.read(&mut buf).unwrap();

            let text = from_utf8(&buf).unwrap();
            println!("Authentication response: {}", text);

            match find_cmd(text) {
                RESP_AUTHENTICATION_SUCCESSFUL => Ok(stream),
                _ => Err("Incorrect username/password"),
            }
        }

        _ => Err("Connection denied"),
    }
}

fn send_cmd(mut stream: &TcpStream, cmd: &str) -> Result<usize, std::io::Error> {
    println!("[UP] {}", cmd);
    stream.write(cmd.as_bytes())
}

// slices a str up to the first semicolon
fn find_cmd(s: &str) -> &str {
    let pos: usize = s.find(";").unwrap();
    &s[..pos]
}

fn login_cmd(user: &str, pass: &str) -> String {
    let user_len = format!("{:0>2}", user.len());
    let pass_len = format!("{:0>2}", pass.len());

    format!("##ID1{}{}{}{};", user_len, pass_len, user, pass)
}

fn sleep(duration: u64) {
    thread::sleep(Duration::from_millis(duration));
}
