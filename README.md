# wol-rs

[![CI](https://github.com/yabowarcherio/wol-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/yabowarcherio/wol-rs/actions/workflows/ci.yml)
[![Crate](https://img.shields.io/crates/v/wol-rs.svg)](https://crates.io/crates/wol-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Build and send **Wake-on-LAN** magic packets in safe Rust. Library + CLI, no
unsafe, no dependencies beyond `clap` for the CLI.

A magic packet is six `0xFF` bytes followed by the target MAC repeated sixteen
times. An optional six-byte SecureOn password may be appended.

## Install

### As a CLI

```sh
cargo install wol-rs
```

### As a library

```toml
[dependencies]
wol-rs = { version = "0.1", default-features = false }   # library-only
```

## Usage (CLI)

```sh
wol-rs 00:11:22:33:44:55                          # broadcast on 255.255.255.255:9
wol-rs --broadcast 192.168.1.255 a4:83:e7:11:22:33
wol-rs --password DE:AD:BE:EF:DE:AD 00:11:22:33:44:55
wol-rs --repeat 3 --interval-ms 200 a4:83:e7:11:22:33   # send 3× with 200 ms pause
wol-rs --dry-run a4:83:e7:11:22:33                # print packet bytes instead
```

Multiple MACs may be given on one invocation; one packet is sent per MAC
(times `--repeat`).

## Usage (library)

```rust
use wol_rs::{magic_packet, magic_packet_with_password, parse_mac};

let mac = parse_mac("a4:83:e7:11:22:33").unwrap();
let pkt = magic_packet(mac);
assert_eq!(pkt.len(), 102);

let with_pw = magic_packet_with_password(mac, [0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD]);
assert_eq!(with_pw.len(), 108);
```

### No-alloc variants

`magic_packet_array` and `magic_packet_with_password_array` return a fixed-size
array on the stack — useful in tight loops, embedded callers, or any path that
needs to avoid the allocator:

```rust
use wol_rs::{magic_packet_array, MAGIC_PACKET_LEN};

const PKT: [u8; MAGIC_PACKET_LEN] = magic_packet_array([0xA4, 0x83, 0xE7, 0x11, 0x22, 0x33]);
assert_eq!(PKT[..6], [0xFF; 6]);
```

Sending it (with the `std` networking stack):

```rust,no_run
use std::net::UdpSocket;
use wol_rs::{magic_packet, parse_mac, BROADCAST_PORT};

let mac = parse_mac("a4:83:e7:11:22:33").unwrap();
let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
sock.set_broadcast(true).unwrap();
sock.send_to(&magic_packet(mac), ("255.255.255.255", BROADCAST_PORT)).unwrap();
```

## Why a magic packet is *just bytes*

The packet itself is layer-2 payload; the destination port is only a
convention. Most receivers listen on UDP `9` (`discard`), some on `7`
(`echo`) — `wol-rs` lets you override the port via `--port`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms.
