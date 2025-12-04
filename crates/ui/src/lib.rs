use annotate_snippets::Group;
use jjpwrgem_parse::{Error, error::diagnostics::Diagnostic};

pub mod pretty;
use crate::pretty::report;

pub fn report_error<'a>(err: &'a Error<'a>) -> Vec<Group<'a>> {
    report(Diagnostic::from(err))
}
