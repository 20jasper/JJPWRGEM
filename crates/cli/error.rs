use displaydoc::Display;
use jjpwrgem_ui::message::BasicErrorMessage;
use thiserror::Error;

use crate::{docs::strip_front_matter, get_docs_snapshot};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Display, Debug, PartialEq, Eq, Clone, Error)]
pub enum Error {
    /// expected non empty input from stdin
    NonEmptyStdinRequired,
}

impl Error {
    fn get_help(&self) -> Option<String> {
        let help = match self {
            Error::NonEmptyStdinRequired => {
                format!(
                    "pipe data to stdin like so\n{}",
                    strip_front_matter(get_docs_snapshot!("check_success"))
                )
            }
        };
        Some(help)
    }
}
impl From<Error> for BasicErrorMessage {
    fn from(value: Error) -> Self {
        let error = value.to_string();
        match value {
            Error::NonEmptyStdinRequired => BasicErrorMessage {
                error,
                help: value.get_help(),
            },
        }
    }
}
