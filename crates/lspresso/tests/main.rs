#![cfg(feature = "test-binaries")]
#![expect(
    clippy::unnecessary_wraps,
    reason = "tests return Result uniformly for ?, even ones that only ever build Ok(())"
)]

#[path = "main/client.rs"]
mod client;
