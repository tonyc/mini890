use std::io::stdout;
use std::str::from_utf8;

pub mod command_callbacks;

use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    spawn,
    sync::mpsc,
    time::{sleep, Duration},
};

use crossterm::{
    cursor, execute,
    style::Print,
    terminal::{Clear, ClearType},
    Result,
};

const RESP_AUTHENTICATION_SUCCESSFUL: &str = "##ID1";
const RESP_CONNECTION_ALLOWED: &str = "##CN1";
const CMD_REQUEST_CONNECTION: &[u8] = b"##CN;";
const CMD_ENABLE_AUTO_INFO: &[u8] = b"AI2;";
const CMD_ENABLE_BANDSCOPE: &[u8] = b"DD01;";
const CMD_DISABLE_BANDSCOPE: &[u8] = b"DD00;";
const CMD_ENABLE_AUDIOSCOPE: &[u8] = b"DD11;";
const CMD_DISABLE_AUDIOSCOPE: &[u8] = b"DD10;";
const RADIO_KEEPALIVE_MS: u64 = 5000;
const CMD_REQUEST_VFO_A: &[u8] = b"FA;";
const CMD_REQUEST_VFO_B: &[u8] = b"FB;";

const HOST: &str = "192.168.1.132:60000";
const USER: &str = "testuser";
const PASS: &str = "testpass123!";

const ENABLE_BANDSCOPE: bool = false;
const ENABLE_AUDIOSCOPE: bool = true;
const MPSC_CHANNEL_SIZE: usize = 64;

//   5   +    1280    + 1
// ##DD2 + [u8: 1280] + ;
const BUFFER_SIZE: usize = 1286; // this appears to be the maximum size of the ##DD2 bandscope message

#[derive(Debug)]
enum Commands {
    PowerStateGet,
}

#[tokio::main]
async fn main() -> Result<()> {
    // execute!(
    //     stdout(),
    //     Clear(ClearType::All),
    //     cursor::DisableBlinking,
    //     cursor::Hide,
    //     cursor::MoveTo(20, 0),
    //     Print("[ mini890 ]")
    // ).unwrap();

    let mut stream: TcpStream = TcpStream::connect(HOST).await?;
    let mut buf = [0 as u8; BUFFER_SIZE];

    stream.write(CMD_REQUEST_CONNECTION).await?;
    stream.read(&mut buf).await?;

    let text = from_utf8(&buf).unwrap();

    match find_cmd(text) {
        RESP_CONNECTION_ALLOWED => {
            stream.write(&login_cmd(USER, PASS).as_bytes()).await?;

            // FIXME: We should probably reset the data buffer each time we read,
            // but we only read once here so it's fine.
            stream.read(&mut buf).await?;

            let text = from_utf8(&buf).unwrap();

            match find_cmd(text) {
                RESP_AUTHENTICATION_SUCCESSFUL => {
                    println!("Authentication successful!");
                    stream.write(CMD_ENABLE_AUTO_INFO).await?;

                    if ENABLE_BANDSCOPE {
                        println!("Bandscope enabled");
                        stream.write(CMD_ENABLE_BANDSCOPE).await?;
                    } else {
                        println!("Bandscope disabled");
                        stream.write(CMD_DISABLE_BANDSCOPE).await?;
                    }

                    if ENABLE_AUDIOSCOPE {
                        println!("Audioscope enabled");
                        stream.write(CMD_ENABLE_AUDIOSCOPE).await?;
                    } else {
                        println!("Audioscope disabled");
                        stream.write(CMD_DISABLE_AUDIOSCOPE).await?;
                    }

                    stream.write(CMD_REQUEST_VFO_A).await?;
                    stream.write(CMD_REQUEST_VFO_B).await?;
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
    let (sender, mut receiver) = mpsc::channel(MPSC_CHANNEL_SIZE);

    let reader_thread = spawn(async move {
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
                        command_callbacks::dispatch(cmd);
                    }

                    // reset the buffer
                    buf.iter_mut().for_each(|x| *x = '0' as u8);
                }
            }
        } // loop

        println!("Client Terminated.");
    });

    let command_writer_thread = spawn(async move {
        println!("writer thread spawned");

        while let Some(cmd) = receiver.recv().await {
            println!("CommandWriterThread: Got cmd: {:?}", cmd);
            match cmd {
                Commands::PowerStateGet => {
                    write_stream.write(b"PS;").await.unwrap();
                }
            }
        }
    });

    let power_watchdog_thread = spawn(async move {
        loop {
            sender.send(Commands::PowerStateGet).await.unwrap();
            sleep(Duration::from_millis(RADIO_KEEPALIVE_MS)).await;
        }
    });

    power_watchdog_thread.await.unwrap();
    reader_thread.await.unwrap();
    command_writer_thread.await.unwrap();

    Ok(())
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
