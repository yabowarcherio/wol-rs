# Security Policy

## Reporting a vulnerability

This crate parses untrusted strings (MAC addresses and SecureOn passwords),
constructs a fixed-size byte buffer, and — in the binary — sends UDP packets
to an operator-supplied destination. It performs no other I/O and embeds no
data. If you nonetheless find a memory-safety or denial-of-service issue,
please report it privately via GitHub's
["Report a vulnerability"](https://github.com/yabowarcherio/wol-rs/security/advisories/new)
flow rather than opening a public issue.

The crate is `#![forbid(unsafe_code)]`, so any soundness bug is necessarily in
a dependency — please include the dependency and version in your report.

## Threat model — what this crate is and isn't

Wake-on-LAN is, by design, an **unauthenticated** L2 protocol: the magic
packet is a UDP payload that triggers a NIC to power on the host whose MAC
appears in the payload. The optional SecureOn password is a six-byte token
appended in plaintext to the packet — it is **not** a cryptographic
authenticator and does not prevent replay. Anyone on the broadcast domain who
can capture one magic packet can resend it.

If you need to wake hosts across an untrusted network, tunnel the WoL agent
inside something authenticated (SSH, WireGuard, etc.). Do not rely on
SecureOn to do that for you.

## Supported versions

The latest released `0.x` line receives fixes.
