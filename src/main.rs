use std::str::from_utf8;
pub mod command_callbacks;

use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
    spawn,
    sync::mpsc,
    time::{sleep, Duration},
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

const ENABLE_BANDSCOPE: bool = true;
const ENABLE_AUDIOSCOPE: bool = false;
const MPSC_CHANNEL_SIZE: usize = 64;

//   5   +    1280    + 1
// ##DD2 + [u8: 1280] + ;
const BUFFER_SIZE: usize = 1286; // this appears to be the maximum size of the ##DD2 bandscope message

#[derive(Debug)]
enum Commands { PowerStateGet, }

#[tokio::main]
async fn main() {
    // execute!(
    //     stdout(),
    //     Clear(ClearType::All),
    //     cursor::DisableBlinking,
    //     cursor::Hide,
    //     cursor::MoveTo(20, 0),
    //     Print("[ mini890 ]")
    // ).unwrap();

    let mut stream: TcpStream = TcpStream::connect(HOST).await.unwrap();
    let mut buf = [0 as u8; BUFFER_SIZE];

    let udp_socket = UdpSocket::bind("0.0.0.0:8000").await.unwrap();
    let remote_addr = "127.0.0.1:8001";
    udp_socket.connect(remote_addr).await.unwrap();


    // let mut buf = [0u8; 32];
    // // recv from remote_addr
    // let len = udp_socket.recv(&mut buf).await?;
    // // send to remote_addr
    // let _len = udp_socket.send(&buf[..len]).await?;

    stream.write(CMD_REQUEST_CONNECTION).await.unwrap();
    stream.read(&mut buf).await.unwrap();

    let text = from_utf8(&buf).unwrap();

    match find_cmd(text) {
        RESP_CONNECTION_ALLOWED => {
            stream.write(&login_cmd(USER, PASS).as_bytes()).await.unwrap();

            // FIXME: We should probably reset the data buffer each time we read,
            // but we only read once here so it's fine.
            stream.read(&mut buf).await.unwrap();

            let text = from_utf8(&buf).unwrap();

            match find_cmd(text) {
                RESP_AUTHENTICATION_SUCCESSFUL => {
                    println!("Authentication successful!");
                    stream.write(CMD_ENABLE_AUTO_INFO).await.unwrap();

                    if ENABLE_BANDSCOPE {
                        println!("Bandscope enabled");
                        stream.write(CMD_ENABLE_BANDSCOPE).await.unwrap();
                    } else {
                        println!("Bandscope disabled");
                        stream.write(CMD_DISABLE_BANDSCOPE).await.unwrap();
                    }

                    if ENABLE_AUDIOSCOPE {
                        println!("Audioscope enabled");
                        stream.write(CMD_ENABLE_AUDIOSCOPE).await.unwrap();
                    } else {
                        println!("Audioscope disabled");
                        stream.write(CMD_DISABLE_AUDIOSCOPE).await.unwrap();
                    }

                    stream.write(CMD_REQUEST_VFO_A).await.unwrap();
                    stream.write(CMD_REQUEST_VFO_B).await.unwrap();
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
    let (tcp_sender, mut tcp_receiver) = mpsc::channel(MPSC_CHANNEL_SIZE);

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
                        command_callbacks::dispatch(cmd, &udp_socket).await.unwrap();
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

        while let Some(cmd) = tcp_receiver.recv().await {
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
            tcp_sender.send(Commands::PowerStateGet).await.unwrap();
            sleep(Duration::from_millis(RADIO_KEEPALIVE_MS)).await;
        }
    });

    power_watchdog_thread.await.unwrap();
    reader_thread.await.unwrap();
    command_writer_thread.await.unwrap();

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
