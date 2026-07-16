// Completes the initialize handshake without spawning IO threads, then sleeps
// forever. Because nothing reads stdin after init, large subsequent client
// writes fill the kernel pipe buffer and block — testing that Transport::drop
// doesn't deadlock when the write thread is stuck.
//
// Must NOT use Connection::stdio(): that spawns a reader thread that keeps
// draining stdin, which would prevent the pipe from filling reliably.
use lspresso::server_helpers as helpers;

fn main() {
    helpers::handle_initialize_no_thread();
    std::thread::sleep(std::time::Duration::MAX);
}
