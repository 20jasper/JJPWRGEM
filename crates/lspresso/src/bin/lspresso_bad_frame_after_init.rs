// Completes the initialize handshake, then sends a frame whose Content-Length
// lies (9999 bytes claimed, far fewer written) before exiting. Used to test
// that the reader thread's break-on-error path surfaces as Disconnected rather
// than Timeout, and does so immediately rather than after the full deadline.
use std::io::Write;

fn main() {
    lspresso::server_helpers::handle_initialize_no_thread();

    let mut stdout = std::io::stdout();
    // Claim 9999 bytes but write only the header — the client's Message::read
    // will try to read 9999 bytes, hit EOF, and return Err.
    write!(stdout, "Content-Length: 9999\r\n\r\n").unwrap();
    stdout.flush().unwrap();
}
