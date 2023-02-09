//use std::io::stdout;

use tokio::net::UdpSocket;
use std::io;
use std::string::String;

//use crossterm::{
//    cursor,
//    execute,
//    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
//};

const BANDSCOPE_BASE: u8 = 140;
const AUDIOSCOPE_BASE: u8 = 50;

pub async fn dispatch(cmd: &str, udp_socket: &UdpSocket) -> io::Result<()> {
    match &cmd[0..=1] {
        "FA" => {
            handle_vfo_a(cmd);
        }
        "FB" => {
            handle_vfo_b(cmd);
        }
        "SM" => {
            handle_smeter(cmd);
        }
        "##" => match &cmd[2..=3] {
            "DD" => match cmd.chars().nth(4) {
                Some('2') => {
                    let bandscope_data = parse_scope(&cmd.replace("##DD2", ""), BANDSCOPE_BASE);
                    // println!("bandscope (len: {}): {:?}", bandscope_data.len(), bandscope_data);

                    let xml_payload = bandscope_to_xml_payload(&bandscope_data);
                    println!("UDP: {}=====\n", xml_payload);
                    udp_socket.send(&xml_payload.as_bytes()).await?;
                }
                Some('3') => {
                    handle_audioscope(cmd);
                }
                _ => {
                    println!("Unknown DD command: {:?}", cmd);
                }
            },

            _ => {
                println!("Unknown LAN command: {:?}", cmd);
            }
        },
        _ => {
            handle_unknown_cmd(cmd);
        }
    }

    Ok(())
}


// TODO: Needs to take scope low and high frequency
pub fn bandscope_to_xml_payload(bandscope_data: &[u8]) -> String {
    let bandscope_data_len = bandscope_data.len();

    let low_scope_freq = 14200000;
    let high_scope_freq = 14350000;

    let spectrum_nums: Vec<String> = bandscope_data.iter()
    .map(|n| n.to_string())
    .collect();

    let spectrum_data: String = spectrum_nums.join(",");

    let xml: String = format!(r###"
<?xml version="1.0" encoding="utf-8"?>
<Spectrum>
    <app>mini890</app>
    <Name>TS-890 Waterfall</Name>
    <LowScopeFrequency>{}</LowScopeFrequency>
    <HighScopeFrequency>{}</HighScopeFrequency>
    <ScalingFactor>1.0</ScalingFactor>
    <DataCount>{}</DataCount>
    <SpectrumData>{}</SpectrumData>
</Spectrum>
"###, low_scope_freq, high_scope_freq, bandscope_data_len, spectrum_data);

    xml
}

pub fn handle_vfo_a(cmd: &str) {
    println!("{}", cmd);
    //execute!(
    //    stdout(),
    //    cursor::MoveTo(0, 1),
    //    SetForegroundColor(Color::White),
    //    SetBackgroundColor(Color::DarkBlue),
    //    Print(format!("A: {}", format_vfo(cmd))),
    //).unwrap();
}

pub fn handle_vfo_b(cmd: &str) {
    println!("{}", cmd);
    //execute!(
    //    stdout(),
    //    cursor::MoveTo(40, 1),
    //    SetForegroundColor(Color::White),
    //    SetBackgroundColor(Color::DarkBlue),
    //    Print(format!("B: {}", format_vfo(cmd))),
    //).unwrap();
}

pub fn handle_smeter(cmd: &str) {
    println!("{}", cmd);
    //execute!(
    //    stdout(),
    //    cursor::MoveTo(0, 5),
    //    SetForegroundColor(Color::White),
    //    SetBackgroundColor(Color::DarkBlue),
    //    Print(format!("Meter: {}", format_smeter(cmd))),
    //).unwrap();
}

pub fn format_vfo(val: &str) -> i32 {
    let hz_str: &str = &val.replace("FA", "").replace("FB", "");

    //let chunks: Vec<char> = hz_str.chars().collect().chunks(3);
    //chunks.join(".")

    //for chunk in hz_str.chars().collect().chunks(3) {
    //    chunks.push(chunk);
    //}

    hz_str.parse::<i32>().unwrap()
}

pub fn format_smeter(val: &str) -> &str {
    val.trim_start_matches("SM0")
}

// pub async fn handle_bandscope(cmd: &str, udp_socket: &UdpSocket) -> Vec<u8> {
//     //eprintln!("Got bandscope length: {}", cmd.len());
//     let bandscope_data = parse_scope(&cmd.replace("##DD2", ""), BANDSCOPE_BASE);
//     println!("bandscope: {:?}", bandscope_data);

//     udp_socket.send(&bandscope_data).await?;

//     bandscope_data
// }

pub fn handle_audioscope(cmd: &str) -> Vec<u8> {
    //println!("Got audioscope length: {}", cmd.len());
    let scope_data = parse_scope(&cmd.replace("##DD3", ""), AUDIOSCOPE_BASE);
    println!("audioscope: {:?}", scope_data);
    scope_data
}

pub fn parse_scope(data: &str, base_value: u8) -> Vec<u8> {
    data.as_bytes()
        .chunks(2)
        .map(|x| std::str::from_utf8(x).unwrap())
        .map(|x| base_value - u8::from_str_radix(x, 16).unwrap())
        //.map(|x| u8::from_str_radix(x, 16).unwrap())
        .collect::<Vec<u8>>()
}

pub fn handle_unknown_cmd(cmd: &str) {
    println!("[DN] {}", cmd);
    //println!("Got audioscope length: {}", cmd.len());
    //Print(format!("[DN] {}", cmd)),
    ()
}
