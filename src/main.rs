//! Command-line interface for `wol-rs`.
//!
//! ```text
//! wol-rs 00:11:22:33:44:55
//! wol-rs --broadcast 192.168.1.255 a4:83:e7:11:22:33
//! wol-rs --password "DE:AD:BE:EF:DE:AD" 00:11:22:33:44:55
//! ```

use std::net::{IpAddr, UdpSocket};
use std::process::ExitCode;

use clap::Parser;
use wol_rs::{magic_packet, magic_packet_with_password, parse_mac, parse_password, BROADCAST_PORT};

/// Send Wake-on-LAN magic packets.
#[derive(Parser, Debug)]
#[command(name = "wol-rs", version, about, long_about = None)]
struct Cli {
    /// Target MAC addresses to wake.
    #[arg(value_name = "MAC", required = true)]
    macs: Vec<String>,

    /// IPv4 broadcast / unicast target address (default `255.255.255.255`).
    #[arg(long, value_name = "ADDR", default_value = "255.255.255.255")]
    broadcast: IpAddr,

    /// UDP port to send to (conventionally `9`).
    #[arg(long, default_value_t = BROADCAST_PORT)]
    port: u16,

    /// Optional SecureOn password (`AA:BB:CC:DD:EE:FF`).
    #[arg(long, value_name = "PASSWORD")]
    password: Option<String>,

    /// Print the packet bytes to stdout instead of sending it.
    #[arg(long, conflicts_with = "broadcast")]
    dry_run: bool,
}

fn run(cli: Cli) -> Result<(), String> {
    let password = match cli.password.as_deref() {
        Some(p) => Some(parse_password(p).map_err(|e| format!("password: {e}"))?),
        None => None,
    };

    let macs: Vec<[u8; 6]> = cli
        .macs
        .iter()
        .map(|s| parse_mac(s).map_err(|e| format!("{s:?}: {e}")))
        .collect::<Result<_, _>>()?;

    if cli.dry_run {
        for mac in &macs {
            let pkt = match password {
                Some(pw) => magic_packet_with_password(*mac, pw),
                None => magic_packet(*mac),
            };
            for b in pkt {
                print!("{b:02X}");
            }
            println!();
        }
        return Ok(());
    }

    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("bind: {e}"))?;
    if cli.broadcast.is_ipv4() && cli.broadcast == IpAddr::from([255u8, 255, 255, 255]) {
        sock.set_broadcast(true)
            .map_err(|e| format!("set_broadcast: {e}"))?;
    }

    for mac in &macs {
        let pkt = match password {
            Some(pw) => magic_packet_with_password(*mac, pw),
            None => magic_packet(*mac),
        };
        sock.send_to(&pkt, (cli.broadcast, cli.port))
            .map_err(|e| format!("send_to: {e}"))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("wol-rs: {e}");
            ExitCode::from(2)
        }
    }
}
