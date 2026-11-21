use std::{borrow::Cow, io};

use gen_lsp_types::*;
use serde::{Deserialize, Deserializer, Serialize, de::IgnoredAny};
use serde_json::value::RawValue;

use crate::lsp_server::req_queue::Id;

#[derive(Serialize, Deserialize)]
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
    pub method: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub params: Option<&'a RawValue>,
}

pub struct Rejection {
    pub id: Id,
    pub code: ErrorCodes,
    pub message: String,
}

fn some<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Id>, D::Error> {
    Id::deserialize(deserializer).map(Some)
}

#[derive(Serialize)]
struct Success<T> {
    jsonrpc: Version,
    id: Id,
    result: T,
}

#[derive(Serialize)]
struct Failure {
    jsonrpc: Version,
    id: Id,
    error: json_rpc::Error,
}

#[derive(Serialize)]
struct Notify<P> {
    jsonrpc: Version,
    method: &'static str,
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
                code: ErrorCodes::InvalidRequest,
                message: format!("invalid request: {error}"),
            }
        } else {
            Rejection {
                id: Id::Null,
                code: ErrorCodes::ParseError,
                message: format!("parse error: {error}"),
            }
        }
    })
}

pub trait Sink {
    fn emit(&mut self, body: &[u8]) -> io::Result<()>;
}

pub struct Writer<S> {
    out: S,
    body: Vec<u8>,
}

impl<S: Sink> Writer<S> {
    pub fn new(out: S) -> Self {
        Self {
            out,
            body: Vec::new(),
        }
    }

    pub fn sink_mut(&mut self) -> &mut S {
        &mut self.out
    }

    pub fn respond<R: Request>(&mut self, id: Id, result: R::Result) -> io::Result<()> {
        self.send(&Success {
            jsonrpc: Version::TwoPointZero,
            id,
            result,
        })
    }

    pub fn fail(&mut self, id: Id, code: ErrorCodes, message: impl Into<String>) -> io::Result<()> {
        let error = json_rpc::Error {
            code,
            message: message.into(),
            data: None,
        };
        self.send(&Failure {
            jsonrpc: Version::TwoPointZero,
            id,
            error,
        })
    }

    pub fn notify<N: Notification>(&mut self, params: N::Params) {
        drop(self.send(&Notify {
            jsonrpc: Version::TwoPointZero,
            method: N::METHOD.as_str(),
            params,
        }));
    }

    fn send(&mut self, message: &impl Serialize) -> io::Result<()> {
        self.body.clear();
        serde_json::to_writer(&mut self.body, message)?;
        self.out.emit(&self.body)
    }
}
