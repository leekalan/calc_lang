use std::fmt::Display;

use parsr::{
    core::trim::TrimWhitespace,
    input::{Entry, Input, InputExt, InvalidUtf8},
    interner::{Id, Interner},
    parse::{IsParse, ParseError, ParseExt, ParseIterError, ParseMutIter},
    token::span::Spanned,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RawToken {
    Ident(Id),
    Number(f64),
    Symbol(Symbol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Equals,
    Add,
    Sub,
    Mul,
    Div,
    LeftParen,
    RightParen,
    Print,
    Semicolon,
}

pub fn parse_raw_tokens<'a: 'b, 'b, I: Input>(
    input: &'a mut I,
    interner: &'b mut Interner,
) -> Result<
    impl Iterator<Item = Result<Spanned<RawToken>, ParseIterError<Spanned<UnexpectedCharacter>>>> + 'b,
    InvalidUtf8,
> {
    let parser = ParseRawToken.mapped_mut(|token: RawTokenInput| {
        let span = match &token {
            RawTokenInput::Alphabetic(entry) => entry.span(),
            RawTokenInput::Numeric(spanned) => spanned.span,
            RawTokenInput::Symbol(spanned) => spanned.span,
        };

        Spanned::new(
            match token {
                RawTokenInput::Alphabetic(entry) => {
                    let id = interner.insert(entry.get());

                    entry.consume();

                    RawToken::Ident(id)
                }
                RawTokenInput::Numeric(num) => RawToken::Number(num.inner),
                RawTokenInput::Symbol(sym) => RawToken::Symbol(sym.inner),
            },
            span,
        )
    });

    ParseMutIter::new(input, TrimWhitespace, parser)
}

pub enum RawTokenInput<'a> {
    Alphabetic(Entry<'a>),
    Numeric(Spanned<f64>),
    Symbol(Spanned<Symbol>),
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseRawToken;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedCharacter;

impl Display for UnexpectedCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected character")
    }
}

impl<'a> IsParse<'a> for ParseRawToken {
    type Output = RawTokenInput<'a>;
    type Error = Spanned<UnexpectedCharacter>;

    fn __parse<I: ?Sized + Input>(
        self,
        input: &'a mut I,
    ) -> Result<Self::Output, parsr::parse::ParseError<Self::Error>> {
        match input.peek()? {
            c if c.is_alphabetic() => {
                let entry = input.read_until_entry(8, |c| !char::is_alphabetic(c))?;

                Ok(RawTokenInput::Alphabetic(entry.unsize()))
            }
            c if c.is_numeric() => {
                let entry = input.read_until_entry(8, |c| !(char::is_numeric(c) || c == '.'))?;

                let num = entry.spanned(
                    entry
                        .get()
                        .parse::<f64>()
                        .map_err(|_| ParseError::new(entry.spanned(UnexpectedCharacter)))?,
                );

                entry.consume();

                Ok(RawTokenInput::Numeric(num))
            }
            _ => {
                let entry = input.peek_entry()?;

                let ret = RawTokenInput::Symbol(entry.spanned(match entry.get() {
                    '=' => Symbol::Equals,
                    '+' => Symbol::Add,
                    '-' => Symbol::Sub,
                    '*' => Symbol::Mul,
                    '/' => Symbol::Div,
                    '(' => Symbol::LeftParen,
                    ')' => Symbol::RightParen,
                    '%' => Symbol::Print,
                    ';' => Symbol::Semicolon,
                    _ => return Err(ParseError::new(entry.spanned(UnexpectedCharacter))),
                }));

                entry.consume();

                Ok(ret)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use parsr::input::StrView;

    use super::*;

    #[test]
    fn test() {
        let mut interner = Interner::new();

        let mut input = StrView::new("a = 1 + 2\n");

        let tokens = parse_raw_tokens(&mut input, &mut interner)
            .unwrap()
            .map(Result::unwrap)
            .map(|r| r.inner)
            .collect::<Vec<_>>();

        let id = interner.insert("a");

        assert_eq!(
            tokens,
            vec![
                RawToken::Ident(id),
                RawToken::Symbol(Symbol::Equals),
                RawToken::Number(1.0),
                RawToken::Symbol(Symbol::Add),
                RawToken::Number(2.0),
            ]
        );
    }
}
