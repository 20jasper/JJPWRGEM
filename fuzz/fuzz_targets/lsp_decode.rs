#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    jjpwrgem_lsp::decode_stream(data);
});
