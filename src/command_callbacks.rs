use std::io::stdout;

use crossterm::{
    cursor,
    execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};

const BANDSCOPE_BASE: u8 = 140;

pub fn dispatch(cmd: &str) {
    match &cmd[0..=1] {
        "FA" => { handle_vfo_a(cmd); }
        "FB" => { handle_vfo_b(cmd); }
        "SM" => { handle_smeter(cmd); }
        "##" => {
            match &cmd[2..=3] {
                "DD" => {
                    match cmd.chars().nth(4) {
                        Some('2') => { handle_bandscope(cmd); }
                        Some('3') => { handle_audioscope(cmd); }
                        _ => { }
                    }

                }

                _ => { println!("Unknown LAN command: {:?}", cmd); }
            }
        }
        _  => { handle_unknown_cmd(cmd) }
    }

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

pub fn handle_bandscope(cmd: &str) -> Vec<u8> {
    //eprintln!("Got bandscope length: {}", cmd.len());
    let bandscope_data = parse_scope(&cmd.replace("##DD2", ""), BANDSCOPE_BASE);
    println!("{:?}", bandscope_data);
    bandscope_data
}

pub fn parse_scope(data: &str, base_value: u8) -> Vec<u8> {
    data
    .as_bytes()
    .chunks(2)
    .map(|x| std::str::from_utf8(x).unwrap())
    .map(|x| base_value - u8::from_str_radix(x, 16).unwrap())
    //.map(|x| u8::from_str_radix(x, 16).unwrap())
    .collect::<Vec<u8>>()
}

pub fn handle_audioscope(cmd: &str) {
    //println!("Got audioscope length: {}", cmd.len());
}

pub fn handle_unknown_cmd(cmd: &str) {
    println!("[DN] {}", cmd);
    //println!("Got audioscope length: {}", cmd.len());
    //Print(format!("[DN] {}", cmd)),
    //()
}
