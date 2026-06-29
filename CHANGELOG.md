# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

Initial release.

### Added

- `magic_packet(mac)` — six `0xFF` bytes followed by the MAC repeated sixteen
  times (the standard 102-byte Wake-on-LAN payload).
- `magic_packet_with_password(mac, password)` — appends a six-byte SecureOn
  password.
- `parse_mac` / `parse_password` accepting colon, hyphen, Cisco-dotted, and
  bare hex spellings, case-insensitively.
- `format_mac` for the canonical upper-case colon form.
- `BROADCAST_PORT` (9) and `ALT_BROADCAST_PORT` (7) constants.
- `wol-rs` CLI with `--broadcast`, `--port`, `--password`, and `--dry-run`.

[Unreleased]: https://github.com/yabowarcherio/wol-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yabowarcherio/wol-rs/releases/tag/v0.1.0
