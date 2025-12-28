use logos::{self, Logos, Skip, SpannedIter};

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexingError {
    NumberParseError,
    #[default]
    Other,
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(_: std::num::ParseIntError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<std::num::ParseFloatError> for LexingError {
    fn from(_: std::num::ParseFloatError) -> Self {
        LexingError::NumberParseError
    }
}

/// Extra data about a token
#[derive(Debug, PartialEq, Default)]
pub struct TokenInfo {
    /// Line number of start
    line: usize,
    /// Byte offset of newline
    line_offset: usize,
    /// Column number of start
    col: usize,
    /// Byte index of start
    start: usize,
    /// Byte index of end
    end: usize,
    /// Number of bytes in token
    len: usize,
}

fn handle_newline(lex: &mut logos::Lexer<Token>) -> Skip {
    lex.extras.line += 1;
    lex.extras.line_offset = lex.span().end;
    Skip
}

fn token_info(lex: &logos::Lexer<Token>) -> TokenInfo {
    TokenInfo {
        line: lex.extras.line,
        line_offset: lex.extras.line_offset,
        col: lex.span().start - lex.extras.line_offset,
        start: lex.span().start,
        end: lex.span().end,
        len: lex.span().end - lex.span().start,
    }
}

fn parse_comment(lex: &logos::Lexer<Token>) -> (String, TokenInfo) {
    (
        lex.slice()[1..lex.slice().len() - 1].to_owned(),
        token_info(lex),
    )
}

fn parse_string(lex: &logos::Lexer<Token>) -> (String, TokenInfo) {
    (
        lex.slice()[2..lex.slice().len()].to_owned(),
        token_info(lex),
    )
}

fn parse_ident(lex: &logos::Lexer<Token>) -> (String, TokenInfo) {
    (lex.slice().to_owned(), token_info(lex))
}

fn parse_int(lex: &logos::Lexer<Token>) -> Result<(i32, TokenInfo), std::num::ParseIntError> {
    Ok((lex.slice().parse()?, token_info(lex)))
}

fn parse_float(lex: &logos::Lexer<Token>) -> Result<(f32, TokenInfo), std::num::ParseFloatError> {
    Ok((lex.slice().parse()?, token_info(lex)))
}

#[derive(Debug, PartialEq, Logos)]
#[logos(extras = TokenInfo)]
#[logos(error = LexingError)]
#[logos(skip(r"\n", handle_newline))]
#[logos(skip(r"[ \t\f]+"))] // Ignore whitespaces between tokens
pub enum Token {
    /// Comments start with "--". Any characters after "--" will be ignored.
    #[regex(r"--.*$", parse_comment, allow_greedy = true)]
    Comment((String, TokenInfo)),
    // String literals use the JSON definition of string literals: https://regex101.com/library/tA9pM8
    #[regex(
        r#""(?:\\(?:["\\/bfnrt]|u[a-fA-F0-9]{4})|[^"\\\x00-\x1F\x7F]+)*""#,
        parse_string
    )]
    StringLiteral((String, TokenInfo)),
    #[regex(r"\d+", parse_int)]
    IntLiteral((i32, TokenInfo)),
    #[regex(r"\d*\.\d+", parse_float)]
    FloatLiteral((f32, TokenInfo)),
    /// Identifiers must be alphanumeric, may contain _'s, and must begin with a letter
    #[regex(r"[a-zA-Z]\w*", parse_ident)]
    Ident((String, TokenInfo)),
    #[token("SELECT", token_info, ignore(case))]
    Select(TokenInfo),
    #[token("FROM", token_info, ignore(case))]
    From(TokenInfo),
    #[token("*", token_info)]
    Star(TokenInfo),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Custom Spanned and Lexer type for use with lalrpop
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        // the Token::lexer() method is provided by the Logos trait
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}
impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream
            .next()
            .map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}
