use crate::{
    Result, SyntaxOptions,
    tokens::{TokenStream, TokenWithContext},
};

/// Tokens read under the syntax a document is allowed to use
pub(crate) struct SyntaxStream<'s, 'a> {
    tokens: &'s mut TokenStream<'a>,
    options: SyntaxOptions,
}

impl<'s, 'a> SyntaxStream<'s, 'a> {
    pub(crate) fn new(tokens: &'s mut TokenStream<'a>, options: SyntaxOptions) -> Self {
        Self { tokens, options }
    }

    #[inline]
    pub(crate) fn options(&self) -> SyntaxOptions {
        self.options
    }

    #[inline]
    pub(crate) fn pos(&self) -> usize {
        self.tokens.pos()
    }

    #[inline]
    pub(crate) fn peek_token(&mut self) -> Result<Option<&TokenWithContext>> {
        self.tokens.peek_token()
    }

    #[inline]
    pub(crate) fn next_token(&mut self) -> Result<Option<TokenWithContext>> {
        self.tokens.next_token()
    }
}
