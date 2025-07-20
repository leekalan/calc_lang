use std::{fmt::Display, iter};

use gxhash::{HashMap, HashMapExt};
use parsr::{interner::Id, token::span::Spanned};

use crate::tokens::{Operator, Token, TokenError, Value};

pub struct State {
    pub variables: HashMap<Id, f64>,
}

impl State {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stack {
    Value(f64),
    Ident(Id),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunErrorContainer {
    TokenError(TokenError),
    RunError(RunError),
}

impl Display for RunErrorContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunErrorContainer::TokenError(e) => write!(f, "{e}"),
            RunErrorContainer::RunError(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunError {
    UnassignedVariable,
    AssigningToExpression,
    AssigningToNull,
    AttemptedToUseNull,
    AttemptedToPrintNull,
    // DivisionByZero,
}

impl Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::UnassignedVariable => write!(f, "Unassigned variable"),
            RunError::AssigningToExpression => write!(f, "Cannot assign to an expression"),
            RunError::AssigningToNull => write!(f, "Cannot assign to NULL"),
            RunError::AttemptedToUseNull => write!(f, "Cannot use NULL"),
            RunError::AttemptedToPrintNull => write!(f, "Cannot print NULL"),
            // RunError::DivisionByZero => write!(f, "Cannot divide by zero"),
        }
    }
}

fn pop_number(
    state: &mut State,
    stack: &mut Vec<Spanned<Stack>>,
) -> Result<Spanned<f64>, Spanned<RunErrorContainer>> {
    let num_stack = stack.pop().unwrap();

    Ok(num_stack.span.over(match num_stack.inner {
        Stack::Value(num) => num,
        Stack::Ident(id) => match state.variables.get(&id) {
            Some(v) => *v,
            None => {
                return Err(num_stack
                    .span
                    .over(RunErrorContainer::RunError(RunError::UnassignedVariable)));
            }
        },
        Stack::Null => {
            return Err(num_stack
                .span
                .over(RunErrorContainer::RunError(RunError::AttemptedToUseNull)));
        }
    }))
}

pub fn run(
    state: &mut State,
    tokens: impl Iterator<Item = Result<Spanned<Token>, Spanned<TokenError>>>,
) -> Result<(), Spanned<RunErrorContainer>> {
    let mut stack = Vec::<Spanned<Stack>>::new();

    let final_semicolon = Ok(Spanned::default_span(Token::Operator(Operator::Semicolon)));

    for token in tokens.chain(iter::once(final_semicolon)) {
        let token = token.map_err(|e| e.map(RunErrorContainer::TokenError))?;

        match token.inner {
            Token::Value(Value::Number(num)) => stack.push(token.span.over(Stack::Value(num))),
            Token::Value(Value::Ident(id)) => stack.push(token.span.over(Stack::Ident(id))),
            Token::Operator(operator) => match operator {
                Operator::Equals => {
                    let num = pop_number(state, &mut stack)?;

                    let var_stack = stack.pop().unwrap();

                    let var = var_stack.span.over(match var_stack.inner {
                        Stack::Ident(id) => id,
                        Stack::Value(_) => {
                            return Err(var_stack.span.over(RunErrorContainer::RunError(
                                RunError::AssigningToExpression,
                            )));
                        }
                        Stack::Null => {
                            return Err(var_stack
                                .span
                                .over(RunErrorContainer::RunError(RunError::AssigningToNull)));
                        }
                    });

                    state.variables.insert(var.inner, num.inner);

                    stack.push(
                        var.span
                            .from_self_to_other(num.span)
                            .over(Stack::Ident(var.inner)),
                    );
                }
                Operator::Add => {
                    let right = pop_number(state, &mut stack)?;
                    let left = pop_number(state, &mut stack)?;

                    stack.push(
                        left.span
                            .from_self_to_other(right.span)
                            .over(Stack::Value(left.inner + right.inner)),
                    );
                }
                Operator::Sub => {
                    let right = pop_number(state, &mut stack)?;
                    let left = pop_number(state, &mut stack)?;

                    stack.push(
                        left.span
                            .from_self_to_other(right.span)
                            .over(Stack::Value(left.inner - right.inner)),
                    );
                }
                Operator::Mul => {
                    let right = pop_number(state, &mut stack)?;
                    let left = pop_number(state, &mut stack)?;

                    stack.push(
                        left.span
                            .from_self_to_other(right.span)
                            .over(Stack::Value(left.inner * right.inner)),
                    );
                }
                Operator::Div => {
                    let right = pop_number(state, &mut stack)?;
                    let left = pop_number(state, &mut stack)?;

                    stack.push(
                        left.span
                            .from_self_to_other(right.span)
                            .over(Stack::Value(left.inner / right.inner)),
                    );
                }
                Operator::Print => {
                    let popped = stack.pop().unwrap();

                    let val = match &popped.inner {
                        Stack::Value(num) => *num,
                        Stack::Ident(id) => match state.variables.get(id) {
                            Some(v) => *v,
                            None => {
                                return Err(popped.span.over(RunErrorContainer::RunError(
                                    RunError::UnassignedVariable,
                                )));
                            }
                        },
                        Stack::Null => {
                            return Err(popped.span.over(RunErrorContainer::RunError(
                                RunError::AttemptedToPrintNull,
                            )));
                        }
                    };

                    print!(" {val}");

                    stack.push(
                        token
                            .span
                            .from_self_to_other(popped.span)
                            .over(popped.inner),
                    );
                }
                Operator::Semicolon => {
                    let popped = stack.pop();

                    if let Some(Spanned {
                        inner: Stack::Ident(id),
                        span,
                    }) = popped.as_ref().map(Spanned::as_ref)
                        && !state.variables.contains_key(id)
                    {
                        return Err(
                            span.over(RunErrorContainer::RunError(RunError::UnassignedVariable))
                        );
                    }

                    let mut span = popped
                        .map(|s| s.span)
                        .unwrap_or_default()
                        .from_self_to_other(token.span);

                    while let Some(Spanned {
                        inner: &Stack::Null,
                        span: earlier_span,
                    }) = stack.last().map(Spanned::as_ref)
                    {
                        span = earlier_span.from_self_to_other(span);

                        let _ = stack.pop();
                    }

                    stack.push(span.over(Stack::Null));
                }
            },
        }
    }

    Ok(())
}
