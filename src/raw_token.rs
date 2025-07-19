use parsr::{
    core::trim::TrimWhitespace,
    input::{Entry, Input, InputExt, InvalidUtf8},
    interner::{Id, Interner},
    parse::{IsParse, ParseError, ParseExt, ParseIterError, ParseMutIter},
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
    impl Iterator<Item = Result<RawToken, ParseIterError<UnexpectedCharacter>>> + 'b,
    InvalidUtf8,
> {
    let parser = ParseRawToken.mapped_mut(|token: RawTokenInput| match token {
        RawTokenInput::Alphabetic(entry) => {
            let id = interner.insert(entry.get());

            entry.consume();

            RawToken::Ident(id)
        }
        RawTokenInput::Numeric(num) => RawToken::Number(num),
        RawTokenInput::Symbol(sym) => RawToken::Symbol(sym),
    });

    ParseMutIter::new(input, TrimWhitespace, parser)
}

pub enum RawTokenInput<'a> {
    Alphabetic(Entry<'a>),
    Numeric(f64),
    Symbol(Symbol),
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseRawToken;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedCharacter;

impl<'a> IsParse<'a> for ParseRawToken {
    type Output = RawTokenInput<'a>;
    type Error = UnexpectedCharacter;

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

                let num = entry
                    .get()
                    .parse::<f64>()
                    .map_err(|_| ParseError::new(UnexpectedCharacter))?;

                entry.consume();

                Ok(RawTokenInput::Numeric(num))
            }
            _ => {
                let entry = input.peek_entry()?;

                let ret = match entry.get() {
                    '=' => RawTokenInput::Symbol(Symbol::Equals),
                    '+' => RawTokenInput::Symbol(Symbol::Add),
                    '-' => RawTokenInput::Symbol(Symbol::Sub),
                    '*' => RawTokenInput::Symbol(Symbol::Mul),
                    '/' => RawTokenInput::Symbol(Symbol::Div),
                    '(' => RawTokenInput::Symbol(Symbol::LeftParen),
                    ')' => RawTokenInput::Symbol(Symbol::RightParen),
                    '%' => RawTokenInput::Symbol(Symbol::Print),
                    ';' => RawTokenInput::Symbol(Symbol::Semicolon),
                    _ => return Err(ParseError::new(UnexpectedCharacter)),
                };

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
