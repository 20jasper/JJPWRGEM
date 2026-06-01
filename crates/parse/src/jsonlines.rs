use crate::{
    Result,
    ast::{Value, parse_at},
    format::{LineEnding, uglify_str_into},
};

#[derive(Debug)]
pub struct JsonlinesDocument<S> {
    source: S,
    values: Vec<Value>,
}

impl<S: AsRef<str>> JsonlinesDocument<S> {
    // jsonlines[impl utf8] — `S: AsRef<str>` is always valid UTF-8 in Rust.
    pub fn parse(source: S) -> Result<Self> {
        let text = source.as_ref();
        // jsonlines[impl end-of-file]
        let text = text.strip_suffix('\n').unwrap_or(text);

        let mut values = Vec::new();
        let mut offset = 0usize;

        // jsonlines[impl newline-delimiter]
        for line in text.split('\n') {
            let r = offset..offset + line.len();

            // jsonlines[impl each-line-is-a-valid-json-value]
            values.push(parse_at(text, r)?.root);

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

pub fn parse(source: &str) -> Result<JsonlinesDocument<&str>> {
    JsonlinesDocument::parse(source)
}

pub fn format(source: &str, line_ending: LineEnding) -> Result<String> {
    let text = source.strip_suffix('\n').unwrap_or(source);
    let mut lines = text.split('\n');

    let Some(line) = lines.next() else {
        return Ok(String::new());
    };

    let mut result = String::new();
    uglify_str_into(&mut result, line)?;

    for line in lines {
        result.push_str(line_ending.as_str());
        uglify_str_into(&mut result, line)?;
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
    fn invalid_lines_fail(#[case] input: &str, #[case] expected_line: usize) {
        let err = parse(input).unwrap_err();
        let line = input[..err.range().start]
            .chars()
            .filter(|&c| c == '\n')
            .count()
            + 1;
        assert_eq!(line, expected_line);
    }
}
