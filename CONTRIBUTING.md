# Contributing to wol-rs

Thanks for considering a contribution! This is a small, focused crate, so the
bar is mostly "keep it tiny, no-deps in the library, and well-tested."

## Getting started

```sh
git clone https://github.com/yabowarcherio/wol-rs
cd wol-rs
cargo test
```

You need a recent stable Rust toolchain (see `rust-version` in
[`Cargo.toml`](Cargo.toml) for the minimum supported version, MSRV).

## Before you open a PR

Please make sure the following all pass locally — CI runs the same checks:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --no-default-features        # library-only must still build (no clap)
cargo deny check                         # licenses & advisories (if installed)
```

Doc-tests too:

```sh
cargo test --doc --all-features
```

## Design constraints

- **The library has zero runtime dependencies.** `clap` is gated behind the
  optional `cli` feature and lives in `src/main.rs` only. Don't pull anything
  else into the library — the value proposition of this crate is "drop into
  any project, compile in milliseconds." If you need an extra dep, gate it
  behind a feature.
- **No `unsafe`.** The crate sets `#![forbid(unsafe_code)]`; keep it that way.
- **No I/O in the library.** Packet construction and parsing only. Sending
  belongs in the binary or in the caller's code.
- **`#![warn(missing_docs)]`.** Every public item gets a doc comment.

## Adding behavior

- Prefer a small free function over a new type. The whole library is six
  free functions plus one error enum on purpose.
- Add unit tests next to the code in `src/lib.rs` for pure logic; add
  integration tests in `tests/` when you want to exercise the binary or the
  public API as a downstream user would.
- Doctest the happy path on any new public function.

## Reporting bugs

Open an issue with the input you used, what you expected, and what happened.
For protocol-level questions (does WoL work across VLAN X, does SecureOn do
what you think) please consult the [SECURITY.md](SECURITY.md) threat model
first — most reports turn out to be about how the protocol itself works.

## Code of Conduct

Be kind and constructive. We follow the spirit of the
[Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
