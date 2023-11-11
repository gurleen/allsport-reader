use std::{io, time::Duration};

use clap::Parser;
use recap::Recap;
use rust_socketio::{client::Client, ClientBuilder};
use serde::Deserialize;
use serde_json::json;
use serialport::SerialPort;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: String,

    #[arg(short, long, default_value = "https://livestats.gurleen.dev")]
    socket_url: String,
}

#[derive(Debug, Deserialize, Recap)]
#[recap(
    regex = r#"(?<game_clock>\d{1,2}:\d{2}(?:\.\d)?)\s*(?:\s*s)?\s*(?<shot_clock>\d{0,2})?\s+(?<home_score>\d+)\s+(?<away_score>\d+)\s+(?<home_fouls>\d+)\s+(?<away_fouls>\d)"#
)]
struct AllSportUpdate {
    game_clock: String,
    shot_clock: String,
    home_score: String,
    away_score: String,
    home_fouls: String,
    away_fouls: String,
}

fn main() {
    let args = Args::parse();
    println!("Connecting COM port {} to {}", args.port, args.socket_url);

    let mut port = serialport::new(args.port, 115_200)
        .timeout(Duration::from_secs(10))
        .open()
        .expect("Failed to open port");

    let mut socket = ClientBuilder::new(args.socket_url)
        .namespace("/")
        .on("error", |err, _| eprintln!("Error: {:#?}", err))
        .connect()
        .expect("Connection failed");

    loop {
        let line = read_line(&mut port).expect("Error reading from port!");
        let parsed: AllSportUpdate = line.parse().expect("Error parsing line!");
        clearscreen::clear().expect("failed to clear screen");
        push_to_socket(&mut socket, parsed);
    }
}

fn read_line(port: &mut Box<dyn SerialPort>) -> io::Result<String> {
    let mut line = String::new();
    let mut buf = [0; 1];

    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(_) => {
                let ch = buf[0] as char;
                if ch == '\x04' {
                    break;
                }
                line.push(ch);
            }
            Err(e) => return Err(e),
        }
    }
    return Ok(line);
}

fn push_to_socket(socket: &mut Client, data: AllSportUpdate) {
    emit_value(socket, "fade:Home-Score", data.home_score);
    emit_value(socket, "fade:Away-Score", data.away_score);
    emit_value(socket, "Clock", data.game_clock);
    emit_value(socket, "Shot-Clock", data.shot_clock);
    emit_value(socket, "Home-Fouls", data.home_fouls);
    emit_value(socket, "Away-Fouls", data.away_fouls);
}

fn emit_value(socket: &mut Client, key: &str, value: String) {
    println!("{} = {}", key.clone(), value.clone());
    match socket.emit("do_update", json!({ key: value })) {
        Ok(_) => return,
        Err(e) => {
            eprintln!("Error emmiting value: {e}");
            return;
        }
    }
}
