//! # wol-rs
//!
//! Build and send **Wake-on-LAN** magic packets.
//!
//! A magic packet is a UDP payload of six `0xFF` bytes followed by the target
//! MAC repeated sixteen times. Optionally, a six-byte SecureOn password may be
//! appended (RFC-less but widely supported by BIOS/UEFI implementations).
//!
//! ```
//! use wol_rs::magic_packet;
//!
//! let mac = [0xA4, 0x83, 0xE7, 0x11, 0x22, 0x33];
//! let pkt = magic_packet(mac);
//! assert_eq!(pkt.len(), 102);
//! assert_eq!(&pkt[..6], &[0xFF; 6]);
//! ```
//!
//! ## Sending the packet
//!
//! On a host with the `std` networking stack:
//!
//! ```no_run
//! use std::net::UdpSocket;
//! use wol_rs::{magic_packet, BROADCAST_PORT};
//!
//! let mac = [0xA4, 0x83, 0xE7, 0x11, 0x22, 0x33];
//! let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
//! sock.set_broadcast(true).unwrap();
//! sock.send_to(&magic_packet(mac), ("255.255.255.255", BROADCAST_PORT)).unwrap();
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

/// The conventional UDP port for Wake-on-LAN broadcasts.
///
/// Magic packets are L2 traffic and the port is only a convention, but
/// `9` (`discard`) is what most software (`wol`, `etherwake`) sends to and
/// what most receivers listen on.
pub const BROADCAST_PORT: u16 = 9;

/// The alternate port used by some receivers (`7`, `echo`).
pub const ALT_BROADCAST_PORT: u16 = 7;

/// Length of a magic packet without a SecureOn password (6 + 6·16 bytes).
pub const MAGIC_PACKET_LEN: usize = 102;

/// Length of a magic packet *with* a SecureOn password (102 + 6 bytes).
pub const MAGIC_PACKET_WITH_PASSWORD_LEN: usize = 108;

/// Build a magic packet for `mac`: six `0xFF` bytes followed by the address
/// repeated sixteen times.
pub fn magic_packet(mac: [u8; 6]) -> Vec<u8> {
    let mut out = Vec::with_capacity(MAGIC_PACKET_LEN);
    out.extend_from_slice(&[0xFF; 6]);
    for _ in 0..16 {
        out.extend_from_slice(&mac);
    }
    out
}

/// Like [`magic_packet`], but appends a six-byte SecureOn password.
pub fn magic_packet_with_password(mac: [u8; 6], password: [u8; 6]) -> Vec<u8> {
    let mut out = magic_packet(mac);
    out.extend_from_slice(&password);
    out
}

/// Errors that can come out of [`parse_mac`] or [`parse_password`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input had fewer than 12 hex digits.
    TooShort,
    /// The input had more than 12 hex digits.
    TooLong,
    /// The input contained a character that wasn't a hex digit or one of the
    /// accepted separators (`:`, `-`, `.`, whitespace).
    InvalidChar(char),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TooShort => f.write_str("too few hex digits (need 12)"),
            ParseError::TooLong => f.write_str("too many hex digits (max 12)"),
            ParseError::InvalidChar(c) => write!(f, "invalid character: {c:?}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a 48-bit MAC address from any common spelling (`:`, `-`, `.`, bare).
pub fn parse_mac(input: &str) -> Result<[u8; 6], ParseError> {
    parse_six_bytes(input)
}

/// Parse a six-byte SecureOn password from any common spelling.
pub fn parse_password(input: &str) -> Result<[u8; 6], ParseError> {
    parse_six_bytes(input)
}

fn parse_six_bytes(input: &str) -> Result<[u8; 6], ParseError> {
    let mut out = [0u8; 6];
    let mut digits = 0usize;
    for c in input.chars() {
        if matches!(c, ':' | '-' | '.') || c.is_whitespace() {
            continue;
        }
        let v = c.to_digit(16).ok_or(ParseError::InvalidChar(c))? as u8;
        if digits >= 12 {
            return Err(ParseError::TooLong);
        }
        let idx = digits / 2;
        out[idx] = (out[idx] << 4) | v;
        digits += 1;
    }
    if digits < 12 {
        return Err(ParseError::TooShort);
    }
    Ok(out)
}

/// Format a six-byte address (MAC or SecureOn password) as canonical
/// `AA:BB:CC:DD:EE:FF`.
pub fn format_mac(bytes: [u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_packet_has_correct_length_and_header() {
        let mac = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let pkt = magic_packet(mac);
        assert_eq!(pkt.len(), MAGIC_PACKET_LEN);
        assert_eq!(&pkt[..6], &[0xFF; 6]);
        for chunk in pkt[6..].chunks(6) {
            assert_eq!(chunk, &mac);
        }
    }

    #[test]
    fn secureon_appends_six_bytes() {
        let pkt = magic_packet_with_password([1; 6], [9; 6]);
        assert_eq!(pkt.len(), MAGIC_PACKET_WITH_PASSWORD_LEN);
        assert_eq!(&pkt[pkt.len() - 6..], &[9; 6]);
    }

    #[test]
    fn parse_mac_accepts_common_spellings() {
        let want = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        for s in [
            "00:11:22:33:44:55",
            "00-11-22-33-44-55",
            "0011.2233.4455",
            "001122334455",
            "00 11 22 33 44 55",
        ] {
            assert_eq!(parse_mac(s).unwrap(), want, "input: {s}");
        }
    }

    #[test]
    fn parse_mac_rejects_short_long_and_garbage() {
        assert_eq!(parse_mac("aa:bb:cc").unwrap_err(), ParseError::TooShort);
        assert_eq!(
            parse_mac("00112233445566").unwrap_err(),
            ParseError::TooLong
        );
        assert_eq!(
            parse_mac("00:11:2g:33:44:55").unwrap_err(),
            ParseError::InvalidChar('g')
        );
    }

    #[test]
    fn format_mac_round_trip() {
        let m = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let s = format_mac(m);
        assert_eq!(s, "AA:BB:CC:DD:EE:FF");
        assert_eq!(parse_mac(&s).unwrap(), m);
    }
}
