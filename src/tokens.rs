use parsr::{
    interner::Id,
    token::{
        span::Spanned,
        token::{
            Associativity, CreateTokenProcessor, FromStackEntry, HasStateTransition, IsOrdering,
            IsResolvedToken, OrderingBehaviour, StackEntry, TokenType,
        },
    },
};

use crate::raw_token::{RawToken, Symbol};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ordering {
    LeftParen,
    RightParen,
    Semicolon,
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TokenTree {
    Null, // TODO
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
            Ordering::Semicolon => OrderingBehaviour::SoftLeft { precedence: 0 },
        }
    }
}

impl FromStackEntry for TokenTree {
    type Token = Token;
    type Ordering = Ordering;

    fn from_entry(_token: &StackEntry<Self::Token, Self::Ordering>) -> Self {
        Self::default()
    }
}

impl Default for TokenTree {
    fn default() -> Self {
        Self::Null
    }
}

impl HasStateTransition<RawToken> for TokenTree {
    type Token = Token;
    type Ordering = Ordering;
    type Error = (); // TODO

    fn transition(
        self,
        token: RawToken,
    ) -> Result<StackEntry<Self::Token, Self::Ordering>, Spanned<Self::Error>> {
        match token {
            RawToken::Ident(id) => Ok(StackEntry::Resolved(Spanned::default_span(Token::Value(
                Value::Ident(id),
            )))),
            RawToken::Number(num) => Ok(StackEntry::Resolved(Spanned::default_span(Token::Value(
                Value::Number(num),
            )))),
            RawToken::Symbol(symbol) => match symbol {
                Symbol::Equals => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Equals),
                ))),
                Symbol::Add => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Add),
                ))),
                Symbol::Sub => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Sub),
                ))),
                Symbol::Mul => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Mul),
                ))),
                Symbol::Div => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Div),
                ))),
                Symbol::LeftParen => Ok(StackEntry::Ordering(Spanned::default_span(
                    Ordering::LeftParen,
                ))),
                Symbol::RightParen => Ok(StackEntry::Ordering(Spanned::default_span(
                    Ordering::RightParen,
                ))),
                Symbol::Print => Ok(StackEntry::Resolved(Spanned::default_span(
                    Token::Operator(Operator::Print),
                ))),
                Symbol::Semicolon => Ok(StackEntry::Ordering(Spanned::default_span(
                    Ordering::Semicolon,
                ))),
            },
        }
    }
}

pub fn resolved_tokens(
    tokens: impl Iterator<Item = RawToken>,
) -> impl Iterator<Item = Result<Spanned<Token>, Spanned<()>>> {
    CreateTokenProcessor::<RawToken, TokenTree, ()>::new(tokens)
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

        let tokens = resolved_tokens(tokens.into_iter());

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
