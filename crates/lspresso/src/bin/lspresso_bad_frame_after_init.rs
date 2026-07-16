// Sends a frame whose Content-Length lies (9999 bytes claimed, none written).
use std::io::Write;

fn main() {
    lspresso::test_server::handle_initialize_no_thread();

    let mut stdout = std::io::stdout();
    write!(stdout, "Content-Length: 9999\r\n\r\n").unwrap();
    stdout.flush().unwrap();
}
