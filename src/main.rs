use std::str::from_utf8;
use std::thread;
use std::time::Duration;
//use std::sync::mpsc;

//use tokio::join;
use tokio::spawn;
use tokio::net::TcpStream;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

const RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
const RESP_CONNECTION_ALLOWED: &str = "##CN1";
const CMD_REQUEST_CONNECTION: &str = "##CN;";

//const CMD_USER_PASS: &str = "##ID10705kenwoodadmin;";
const HOST: &str = "192.168.1.229:60000";
const USER: &str = "testuser";
const PASS: &str = "testpass123!";
const BUFFER_SIZE: usize = 1286; // this appears to be the maximum size of the ##DD2 bandscope message

//#[derive(Debug)]
//enum Commands {
//    PowerState
//}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream: TcpStream = TcpStream::connect(HOST).await?;
    println!("Connected to server on port 1234");

    stream.write(CMD_REQUEST_CONNECTION.as_bytes()).await?;
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
                    stream.write("AI2;".as_bytes()).await?;

                    // enable bandscope
                    //stream.write("DD11;".as_bytes()).await?;
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
    let (tx, mut rx) = mpsc::channel(32);

    let reader_thread = spawn(async move {
        println!("spawning connection thread");

        let mut buf = [' ' as u8; BUFFER_SIZE];
        loop {
            match read_stream.read(&mut buf).await.unwrap() {
                0 => {
                    println!("No bytes to read. Did the radio drop the connection?");
                    break;
                }

                n => {
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

                //Err(other) => {
                //    println!("Error reading stream: {:?}", other);
                //    break;
                //}
            }

            thread::yield_now();
        } // loop

        println!("Client Terminated.");
    });

    let writer_thread = spawn(async move {
        println!("writer thread spawned");

        while let Some(cmd) = rx.recv().await {
            println!("Got cmd: {:?}", cmd);
            write_stream.write_all(cmd).await.unwrap();
        }

        //loop {
        //    match rx.recv().await {
        //        Some(cmd) => {
        //            println!("Got something: {:?}", cmd);
        //            write_stream.write_all(cmd.as_bytes()).await.unwrap();
        //        }
        //        None => {
        //            println!("Got nothing");
        //        }
        //    }
        //    //println!("recv loop");
        //    ////println!("received cmd: {:?}", cmd);

        //    //let cmd = rx.recv().;
        //    ////write_stream.write(cmd.as_bytes()).await.unwrap();
        //}

        //while let Some(cmd) = rx.recv().await {
        //    println!("received cmd: {:?}", cmd);
        //    write_stream.write(cmd.as_bytes()).await.unwrap();
        //}
        //println!("Writer thread done");

    });

    let timer_thread = spawn(async move {
        println!("spawning timer thread");
        loop {
            println!("pinging radio");
            tx.send("PS;".as_bytes()).await.unwrap();
            sleep(500);
        }
    });


    timer_thread.await.unwrap();
    reader_thread.await.unwrap();
    writer_thread.await.unwrap();

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

//fn send_cmd(stream: &TcpStream, cmd: &str) {
//    println!("[UP] {}", cmd);
//    stream.write(cmd.as_bytes());
//}

//fn send_cmd(write_half: WriteHalf<TcpStream>, cmd: &str) {
//    println!("[UP] {}", cmd);
//    write_half.write(cmd.as_bytes());
//}

//async fn send_cmd_async(mut stream: WriteHalf<TcpStream>, cmd: &str) -> Result<usize, std::io::Error> {
//    println!("[UP] {}", cmd);
//    let bytes: usize = stream.write(cmd.as_bytes()).await.unwrap();

//    Ok(bytes)
//}

//fn send_cmd(mut stream: tokio::net::TcpStream, cmd: &str) -> Result<usize, _> {
//    println!("[UP] {}", cmd);
//    stream.write(cmd.as_bytes()).await.unwrap()
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

fn sleep(duration: u64) {
    thread::sleep(Duration::from_millis(duration));
}
