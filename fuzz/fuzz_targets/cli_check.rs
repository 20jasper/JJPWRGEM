#![no_main]

use jjpwrgem_parse::{
    diagnostics::Diagnostic,
    format::{LineEnding, prettify_str, uglify_str},
    validate_str,
};
use jjpwrgem_ui::{Color, Style};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let style = Style::Pretty(Color::Plain);

    let s = match std::str::from_utf8(data) {
        Err(e) => {
            let err = jjpwrgem_parse::Error::from_utf8_error_slice(e, data);
            let _ = style.render_diagnostic(Diagnostic::from(&err));
            return;
        }
        Ok(s) => s,
    };

    match validate_str(s) {
        Ok(()) => {}
        Err(err) => {
            let _ = style.render_diagnostic(Diagnostic::from(&err));
        }
    }

    match prettify_str(s, 80, LineEnding::Lf) {
        Ok(_) => {}
        Err(err) => {
            let _ = style.render_diagnostic(Diagnostic::from(&err));
        }
    }

    match uglify_str(s) {
        Ok(_) => {}
        Err(err) => {
            let _ = style.render_diagnostic(Diagnostic::from(&err));
        }
    }
});
