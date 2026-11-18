# Third-party notices

## lsp-server

- Files: `crates/lsp/src/vendored.rs`, `crates/lsp/src/stdio.rs`, `crates/lsp/src/req_queue.rs`
- Origin: rust-lang/rust-analyzer, `lib/lsp-server` (version 0.10.0), `src/msg.rs`, `src/stdio.rs` and `src/req_queue.rs`
- Vendored: `read_msg_text` and `write_msg_text` (Content-Length framing), adapted to reuse caller-owned buffers, read the body without trusting the declared length, and work on bytes
- Vendored: `ErrorCode` (used variants only) and `ResponseError` (serialize only)
- Vendored: the stdin reader thread from `stdio_transport`, sending owned message bytes and read errors over `std::sync::mpsc` instead of crossbeam, with no writer or dropper thread
- Vendored: the outgoing half of `ReqQueue`, keyed by the language server types `Id`. The incoming half is not vendored, since cancellation has nothing to cancel on a single thread that answers each request before reading the next message
- Copyright: rust-analyzer contributors
- License: MIT OR Apache-2.0. The MIT text is reproduced below; the Apache-2.0 text is at https://github.com/rust-lang/rust-analyzer/blob/master/LICENSE-APACHE

### MIT License

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
