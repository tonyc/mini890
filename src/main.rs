use std::str::from_utf8;

use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

const RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
const RESP_CONNECTION_ALLOWED: &str = "##CN1";
const CMD_REQUEST_CONNECTION: &[u8] = b"##CN;";
const CMD_ENABLE_AUTO_INFO: &[u8] = b"AI2;";
const CMD_ENABLE_BANDSCOPE: &[u8] = b"DD01;";
const RADIO_KEEPALIVE_MS: u64 = 5000;

const HOST: &str = "192.168.1.229:60000";
const USER: &str = "testuser";
const PASS: &str = "testpass123!";

const ENABLE_BANDSCOPE: bool = false;
const MPSC_CHANNEL_SIZE: usize = 64;

//   5   +    1280    + 1
// ##DD2 + [u8: 1280] + ;
const BUFFER_SIZE: usize = 1286; // this appears to be the maximum size of the ##DD2 bandscope message

#[derive(Debug)]
enum Commands {
    PowerStateGet,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream: TcpStream = TcpStream::connect(HOST).await?;
    println!("Connected to server on port 1234");

    stream.write(CMD_REQUEST_CONNECTION).await?;
    println!("Sent connection request CN, awaiting reply...");

    let mut buf = [0 as u8; BUFFER_SIZE];
    stream.read(&mut buf).await?;

    let text = from_utf8(&buf).unwrap();

    // BUG: text has the entire 1k buffer padded with zeroes
    println!("Read text: {}", text);

    match find_cmd(text) {
        RESP_CONNECTION_ALLOWED => {
            println!("Sending username/password");
            stream.write(&login_cmd(USER, PASS).as_bytes()).await?;
            //send_cmd(&stream, &login_cmd(USER, PASS));

            // TODO: We should probably reset the data buffer each time we read
            stream.read(&mut buf).await?;

            let text = from_utf8(&buf).unwrap();
            println!("Authentication response: {}", text);

            match find_cmd(text) {
                RESP_AUTHENTICATION_SUCCESSFUL => {
                    println!("Authentication successful!");
                    stream.write(CMD_ENABLE_AUTO_INFO).await?;

                    // enable bandscope - high cycle lan
                    if ENABLE_BANDSCOPE {
                        stream.write(CMD_ENABLE_BANDSCOPE).await?;
                    }
                }
                other => {
                    println!("Unknown command: {:?}", other);
                }
            }
        }
        other => {
            println!("Unknown command: {:?}", other);
        }
    }

    let (mut read_stream, mut write_stream) = split(stream);
    let (tx, mut rx) = mpsc::channel(MPSC_CHANNEL_SIZE);

    let reader_thread = spawn(async move {
        println!("spawning connection thread");

        let mut buf = ['0' as u8; BUFFER_SIZE];
        loop {
            match read_stream.read(&mut buf).await.unwrap() {
                0 => {
                    println!("No bytes to read. Did the radio drop the connection?");
                    break;
                }

                n => {
                    let text = from_utf8(&buf[0..n]).unwrap();

                    for cmd in text.split_terminator(";") {
                        println!("[DN] {}", cmd);
                    }

                    // reset the buffer
                    buf.iter_mut().for_each(|x| *x = '0' as u8);
                }
            }
        } // loop

        println!("Client Terminated.");
    });

    //let writer_thread = spawn(async move {
    //    println!("writer thread spawned");

    //    while let Some(cmd) = rx.recv().await {
    //        println!("Got cmd: {:?}", cmd);
    //        write_stream.write_all(cmd).await.unwrap();
    //    }

    //});

    let timer_thread = spawn(async move {
        println!("spawning timer thread");
        loop {
            println!("Pinging radio");
            tx.send(Commands::PowerStateGet).await.unwrap();
            sleep(Duration::from_millis(RADIO_KEEPALIVE_MS)).await;
        }
    });


    println!("entering receive loop");
    while let Some(cmd) = rx.recv().await {
        println!("Got cmd: {:?}", cmd);

        match cmd {
            Commands::PowerStateGet => {
                println!("keeping radio alive with PS;");
                write_stream.write(b"PS;").await.unwrap();
            }
        }
    }

    timer_thread.await.unwrap();
    reader_thread.await.unwrap();
    //writer_thread.await.unwrap();

    Ok(())
}


//async fn radio_authenticate(mut read_half: ReadHalf<TcpStream>, mut write_half: WriteHalf<TcpStream>) -> Result<(), &'static str> {
//    send_cmd_async(write_half, &CMD_REQUEST_CONNECTION).await.unwrap();
//    println!("Sent connection request CN, awaiting reply...");

//    let mut buf = [0 as u8; BUFFER_SIZE];
//    // let data: Vec<u8> = vec![];

//    read_half.read(&mut buf).await.unwrap();

//    let text = from_utf8(&buf).unwrap();

//    // BUG: text has the entire 1k buffer padded with zeroes
//    println!("Read text: {}", text);

//    match find_cmd(text) {
//        RESP_CONNECTION_ALLOWED => {
//            println!("Sending username/password");
//            send_cmd_async(write_half, &login_cmd(USER, PASS)).await.unwrap();

//            // BUG: We should probably reset the data buffer each time we read
//            read_half.read(&mut buf).await.unwrap();

//            let text = from_utf8(&buf).unwrap();
//            println!("Authentication response: {}", text);

//            match find_cmd(text) {
//                RESP_AUTHENTICATION_SUCCESSFUL => Ok(()),
//                _ => Err("Incorrect username/password"),
//            }
//        }

//        _ => Err("Connection denied"),
//    }
//}

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

