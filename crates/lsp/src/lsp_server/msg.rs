// vendored from lsp-server v0.10.0
use std::io::{self, BufRead, Read, Write};

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

pub fn read_msg_text(
    inp: &mut dyn BufRead,
    buf: &mut String,
    body: &mut Vec<u8>,
) -> io::Result<bool> {
    let mut size = None;
    loop {
        buf.clear();
        if inp.read_line(buf)? == 0 {
            return Ok(false);
        }
        if !buf.ends_with("\r\n") {
            return Err(invalid_data(format!("malformed header: {buf:?}")));
        }
        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }
        let mut parts = buf.splitn(2, ": ");
        let header_name = parts.next().unwrap();
        let header_value = parts
            .next()
            .ok_or_else(|| invalid_data(format!("malformed header: {buf:?}")))?;
        if header_name.eq_ignore_ascii_case("Content-Length") {
            size = Some(header_value.parse::<usize>().map_err(invalid_data)?);
        }
    }
    let size: usize = size.ok_or_else(|| invalid_data("no Content-Length"))?;
    body.clear();
    if inp.take(size as u64).read_to_end(body)? < size {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(true)
}

pub fn write_msg_text(out: &mut dyn Write, msg: &[u8]) -> io::Result<()> {
    write!(out, "Content-Length: {}\r\n\r\n", msg.len())?;
    out.write_all(msg)?;
    out.flush()?;
    Ok(())
}
