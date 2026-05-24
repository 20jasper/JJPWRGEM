#![no_main]

use jjpwrgem_parse::format::{LineEnding, prettify_str};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let _ = prettify_str(s, 80, LineEnding::Lf);
});
