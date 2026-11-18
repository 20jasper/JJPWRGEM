use std::{
    borrow::Cow,
    io::{self, BufRead, BufWriter, StdoutLock, stdout},
};

use gen_lsp_types::*;
use serde::{Deserialize, Deserializer, Serialize, de::IgnoredAny};
use serde_json::value::RawValue;

use crate::lsp_server::{
    msg::{ErrorCode, ResponseError, read_msg_text, write_msg_text},
    req_queue::Id,
};

const JSONRPC: &str = "2.0";

#[derive(Deserialize)]
enum Version {
    #[serde(rename = "2.0")]
    TwoPointZero,
}

#[derive(Deserialize)]
pub struct Incoming<'a> {
    #[expect(dead_code, reason = "decoding rejects other protocol versions")]
    jsonrpc: Version,
    #[serde(default, deserialize_with = "some")]
    pub id: Option<Id>,
    #[serde(default, borrow)]
    pub method: Cow<'a, str>,
    #[serde(default, borrow)]
    pub params: Option<&'a RawValue>,
}

pub struct Rejection {
    pub id: Id,
    pub code: ErrorCode,
    pub message: String,
}

fn some<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Id>, D::Error> {
    Id::deserialize(deserializer).map(Some)
}

#[derive(Serialize)]
struct Success<T> {
    jsonrpc: &'static str,
    id: Id,
    result: T,
}

#[derive(Serialize)]
struct Failure {
    jsonrpc: &'static str,
    id: Id,
    error: ResponseError,
}

#[derive(Serialize)]
struct Notify<P> {
    jsonrpc: &'static str,
    method: String,
    params: P,
}

pub fn decode(body: &[u8]) -> Result<Incoming<'_>, Rejection> {
    serde_json::from_slice(body).map_err(|error| {
        if serde_json::from_slice::<IgnoredAny>(body).is_ok() {
            Rejection {
                id: serde_json::from_slice::<serde_json::Value>(body)
                    .ok()
                    .and_then(|value| Id::deserialize(value.get("id")?).ok())
                    .unwrap_or(Id::Null),
                code: ErrorCode::InvalidRequest,
                message: format!("invalid request: {error}"),
            }
        } else {
            Rejection {
                id: Id::Null,
                code: ErrorCode::ParseError,
                message: format!("parse error: {error}"),
            }
        }
    })
}

pub struct Reader<R> {
    input: R,
    line: String,
    body: Vec<u8>,
}

impl<R: BufRead> Reader<R> {
    pub fn from_input(input: R) -> Self {
        Self {
            input,
            line: String::new(),
            body: Vec::new(),
        }
    }

    pub fn message(&self) -> Result<Incoming<'_>, Rejection> {
        decode(&self.body)
    }

    pub fn read_body(&mut self) -> io::Result<bool> {
        read_msg_text(&mut self.input, &mut self.line, &mut self.body)
    }
}

pub struct Writer {
    out: BufWriter<StdoutLock<'static>>,
    body: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            out: BufWriter::new(stdout().lock()),
            body: Vec::new(),
        }
    }

    pub fn respond<R: Request>(&mut self, id: Id, result: R::Result) -> io::Result<()> {
        self.send(&Success {
            jsonrpc: JSONRPC,
            id,
            result,
        })
    }

    pub fn fail(&mut self, id: Id, code: ErrorCode, message: String) -> io::Result<()> {
        let error = ResponseError {
            code: code as i32,
            message,
            data: None,
        };
        self.send(&Failure {
            jsonrpc: JSONRPC,
            id,
            error,
        })
    }

    pub fn notify<N: Notification>(&mut self, params: N::Params) {
        drop(self.send(&Notify {
            jsonrpc: JSONRPC,
            method: N::METHOD.to_string(),
            params,
        }));
    }

    fn send(&mut self, message: &impl Serialize) -> io::Result<()> {
        self.body.clear();
        serde_json::to_writer(&mut self.body, message)?;
        write_msg_text(&mut self.out, &self.body)
    }
}
