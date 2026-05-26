use crate::{
    Error,
    ast::{Value, parse_at},
    format::uglify_str_into,
};

#[derive(Debug)]
pub struct JsonlinesParseError {
    pub line: usize,
    pub error: Error,
}

#[derive(Debug)]
pub struct JsonlinesDocument<S> {
    source: S,
    values: Vec<Value>,
}

impl<S: AsRef<str>> JsonlinesDocument<S> {
    // jsonlines[impl utf8] — `S: AsRef<str>` is always valid UTF-8 in Rust.
    pub fn parse(source: S) -> Result<Self, JsonlinesParseError> {
        let text = source.as_ref();
        // jsonlines[impl end-of-file]
        let text = text.strip_suffix('\n').unwrap_or(text);

        let mut values = Vec::new();
        let mut offset = 0usize;

        // jsonlines[impl newline-delimiter]
        for (i, line) in text.split('\n').enumerate() {
            let r = offset..offset + line.len();

            // jsonlines[impl each-line-is-a-valid-json-value]
            let doc =
                parse_at(text, r).map_err(|error| JsonlinesParseError { line: i + 1, error })?;
            values.push(doc.root);

            offset = r.end + 1;
        }

        Ok(Self { source, values })
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub fn parse(source: &str) -> Result<JsonlinesDocument<&str>, JsonlinesParseError> {
    JsonlinesDocument::parse(source)
}

pub fn format(source: &str) -> Result<String, JsonlinesParseError> {
    let text = source.strip_suffix('\n').unwrap_or(source);
    let mut lines = text.split('\n').enumerate();

    let Some((i, line)) = lines.next() else {
        return Ok(String::new());
    };

    let mut result = String::new();
    uglify_str_into(&mut result, line)
        .map_err(|error| JsonlinesParseError { line: i + 1, error })?;

    for (i, line) in lines {
        result.push('\n');
        uglify_str_into(&mut result, line)
            .map_err(|error| JsonlinesParseError { line: i + 1, error })?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    // jsonlines[verify newline-delimiter]
    #[rstest]
    #[case(r#"{"a":1}"#, 1)]
    #[case("{\"a\":1}\n{\"b\":2}", 2)]
    fn lines_split_on_newline(#[case] input: &str, #[case] count: usize) {
        assert_eq!(parse(input).unwrap().len(), count);
    }

    // jsonlines[verify end-of-file]
    #[rstest]
    #[case("{\"a\":1}\n", 1)]
    #[case("{\"a\":1}\n{\"b\":2}\n", 2)]
    fn trailing_newline_is_allowed(#[case] input: &str, #[case] count: usize) {
        assert_eq!(parse(input).unwrap().len(), count);
    }

    // jsonlines[verify each-line-is-a-valid-json-value]
    #[rstest]
    #[case("\n{\"a\":1}", 1)]
    #[case("{\"a\":1}\n\n{\"b\":2}", 2)]
    #[case("{\"a\":1}\n{bad}", 2)]
    fn invalid_lines_fail(#[case] input: &str, #[case] error_line: usize) {
        assert_eq!(parse(input).unwrap_err().line, error_line);
    }
}
