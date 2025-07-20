use std::fmt::Display;

use parsr::{
    interner::Id,
    parse::ParseIterError,
    token::{
        span::Spanned,
        token::{
            Associativity, CreateTokenProcessor, FromStackEntry, HasStateTransition, IsOrdering,
            IsResolvedToken, IsState, OrderingBehaviour, StackEntry, TokenType,
        },
    },
};

use crate::raw_token::{RawToken, Symbol, UnexpectedCharacter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenError {
    RawToken(ParseIterError<UnexpectedCharacter>),
    ProcessorError(ProcessorError),
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::RawToken(e) => write!(f, "{e}"),
            TokenError::ProcessorError(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessorError {
    ExpectedExpression,
    DidNotExpectExpression,
    UnclosedLeftBracket,
    UnclosedRightBracket,
}

impl Display for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessorError::ExpectedExpression => write!(f, "Expected expression"),
            ProcessorError::DidNotExpectExpression => write!(f, "Did not expect expression"),
            ProcessorError::UnclosedLeftBracket => write!(f, "Unclosed left bracket"),
            ProcessorError::UnclosedRightBracket => write!(f, "Unclosed right bracket"),
        }
    }
}

impl From<ParseIterError<UnexpectedCharacter>> for TokenError {
    #[inline(always)]
    fn from(value: ParseIterError<UnexpectedCharacter>) -> Self {
        TokenError::RawToken(value)
    }
}

impl From<ProcessorError> for TokenError {
    fn from(value: ProcessorError) -> Self {
        TokenError::ProcessorError(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Value(Value),
    Operator(Operator),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Number(f64),
    Ident(Id),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Equals,
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Semicolon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ordering {
    LeftParen,
    RightParen,
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TokenTree {
    StartExpression,
    EndExpression,
}

pub struct State {}

#[allow(clippy::derivable_impls)]
impl Default for State {
    fn default() -> Self {
        Self {}
    }
}

impl IsState<Token, Ordering> for State {
    type Error = ProcessorError;

    fn update(&mut self, _token: &StackEntry<Token, Ordering>) {}

    fn delete_closed_operator(&mut self, _: &mut Spanned<Token>) -> Option<Spanned<Self::Error>> {
        None
    }

    fn delete_closed_ordering(
        &mut self,
        ordering: Spanned<Ordering>,
    ) -> Option<Spanned<Self::Error>> {
        Some(ordering.span.over(ProcessorError::UnclosedRightBracket))
    }

    fn no_ordering_found(&self, ordering: Spanned<Ordering>) -> Option<Spanned<Self::Error>> {
        Some(ordering.span.over(ProcessorError::UnclosedLeftBracket))
    }
}

impl IsResolvedToken for Token {
    fn get_type(&self) -> TokenType {
        match self {
            Token::Value(_) => TokenType::Value,
            Token::Operator(operator) => IsResolvedToken::get_type(operator),
        }
    }
}

impl IsResolvedToken for Operator {
    fn get_type(&self) -> TokenType {
        match self {
            Operator::Equals => TokenType::Precedence {
                precedence: 1,
                associativity: Associativity::Right,
            },
            Operator::Add | Operator::Sub => TokenType::Precedence {
                precedence: 2,
                associativity: Associativity::Left,
            },
            Operator::Mul | Operator::Div => TokenType::Precedence {
                precedence: 3,
                associativity: Associativity::Left,
            },
            Operator::Print => TokenType::Precedence {
                precedence: 4,
                associativity: Associativity::Right,
            },
            Operator::Semicolon => TokenType::Precedence {
                precedence: 0,
                associativity: Associativity::Left,
            },
        }
    }
}

impl IsOrdering for Ordering {
    fn behaviour(&self) -> OrderingBehaviour {
        match self {
            Ordering::LeftParen => OrderingBehaviour::Right {
                precedence: 5,
                closed: true,
            },
            Ordering::RightParen => OrderingBehaviour::ClosedLeft,
        }
    }
}

impl FromStackEntry for TokenTree {
    type Token = Token;
    type Ordering = Ordering;

    fn from_entry(token: &StackEntry<Self::Token, Self::Ordering>) -> Self {
        match token {
            StackEntry::Resolved(t) => match t.inner {
                Token::Value(_) => Self::EndExpression,
                Token::Operator(_) => Self::StartExpression,
            },
            StackEntry::Ordering(t) => match t.inner {
                Ordering::LeftParen => Self::StartExpression,
                Ordering::RightParen => Self::EndExpression,
            },
        }
    }
}

impl Default for TokenTree {
    fn default() -> Self {
        Self::StartExpression
    }
}

impl HasStateTransition<Spanned<RawToken>> for TokenTree {
    type Token = Token;
    type Ordering = Ordering;
    type Error = ProcessorError;

    fn transition(
        self,
        token: Spanned<RawToken>,
    ) -> Result<StackEntry<Self::Token, Self::Ordering>, Spanned<Self::Error>> {
        match self {
            TokenTree::StartExpression => match token.inner {
                RawToken::Ident(id) => Ok(StackEntry::Resolved(Spanned::new(
                    Token::Value(Value::Ident(id)),
                    token.span,
                ))),
                RawToken::Number(num) => Ok(StackEntry::Resolved(Spanned::new(
                    Token::Value(Value::Number(num)),
                    token.span,
                ))),
                RawToken::Symbol(symbol) => match symbol {
                    Symbol::LeftParen => Ok(StackEntry::Ordering(Spanned::new(
                        Ordering::LeftParen,
                        token.span,
                    ))),
                    Symbol::Print => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Print),
                        token.span,
                    ))),
                    Symbol::Semicolon => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Semicolon),
                        token.span,
                    ))),
                    _ => Err(token.span.over(ProcessorError::ExpectedExpression)),
                },
            },
            TokenTree::EndExpression => match token.inner {
                RawToken::Symbol(symbol) => match symbol {
                    Symbol::Equals => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Equals),
                        token.span,
                    ))),
                    Symbol::Add => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Add),
                        token.span,
                    ))),
                    Symbol::Sub => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Sub),
                        token.span,
                    ))),
                    Symbol::Mul => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Mul),
                        token.span,
                    ))),
                    Symbol::Div => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Div),
                        token.span,
                    ))),
                    Symbol::RightParen => Ok(StackEntry::Ordering(Spanned::new(
                        Ordering::RightParen,
                        token.span,
                    ))),
                    Symbol::Semicolon => Ok(StackEntry::Resolved(Spanned::new(
                        Token::Operator(Operator::Semicolon),
                        token.span,
                    ))),
                    _ => Err(token.span.over(ProcessorError::DidNotExpectExpression)),
                },
                _ => Err(token.span.over(ProcessorError::DidNotExpectExpression)),
            },
        }
    }
}

pub fn resolved_tokens(
    tokens: impl Iterator<
        Item = Result<Spanned<RawToken>, ParseIterError<Spanned<UnexpectedCharacter>>>,
    >,
) -> impl Iterator<Item = Result<Spanned<Token>, Spanned<TokenError>>> {
    CreateTokenProcessor::<Spanned<RawToken>, TokenTree, State, TokenError>::new(
        tokens.map(|r| r.map_err(|e| e.spanned().map(TokenError::RawToken))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let tokens = [
            RawToken::Ident(unsafe { Id::from_usize(0) }),
            RawToken::Symbol(Symbol::Equals),
            RawToken::Number(1.0),
            RawToken::Symbol(Symbol::Add),
            RawToken::Number(2.0),
        ];

        let tokens = resolved_tokens(
            tokens
                .into_iter()
                .map(Spanned::default_span)
                .map(Result::Ok),
        );

        assert_eq!(
            tokens.collect::<Vec<_>>(),
            vec![
                Ok(Spanned::default_span(Token::Value(Value::Ident(unsafe {
                    Id::from_usize(0)
                })))),
                Ok(Spanned::default_span(Token::Value(Value::Number(1.0)))),
                Ok(Spanned::default_span(Token::Value(Value::Number(2.0)))),
                Ok(Spanned::default_span(Token::Operator(Operator::Add))),
                Ok(Spanned::default_span(Token::Operator(Operator::Equals))),
            ]
        );
    }
}
