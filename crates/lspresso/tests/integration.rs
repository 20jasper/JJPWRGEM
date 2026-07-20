#![cfg(feature = "test-binaries")]
#![expect(
    clippy::unnecessary_wraps,
    reason = "tests return Result uniformly for ?, even ones that only ever build Ok(())"
)]

#[path = "integration/client.rs"]
mod client;
#[path = "integration/diagnostics.rs"]
mod diagnostics;
#[path = "integration/errors.rs"]
mod errors;
#[path = "integration/support.rs"]
mod support;
