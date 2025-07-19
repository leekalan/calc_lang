use gxhash::{HashMap, HashMapExt};
use parsr::interner::Id;

use crate::tokens::{Operator, Token, Value};

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

pub enum StackValue {
    Value(f64),
    Ident(Id),
}

pub fn run(state: &mut State, tokens: impl Iterator<Item = Token>) {
    let mut stack = Vec::<StackValue>::new();

    for token in tokens {
        match token {
            Token::Value(Value::Number(num)) => stack.push(StackValue::Value(num)),
            Token::Value(Value::Ident(id)) => stack.push(StackValue::Ident(id)),
            Token::Operator(operator) => match operator {
                Operator::Equals => {
                    let num = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    let var = match stack.pop().unwrap() {
                        StackValue::Ident(id) => id,
                        _ => unreachable!("cannot assign to a number"),
                    };

                    state.variables.insert(var, num);
                }
                Operator::Add => {
                    let right = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    let left = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    stack.push(StackValue::Value(left + right));
                }
                Operator::Sub => {
                    let right = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    let left = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    stack.push(StackValue::Value(left - right));
                }
                Operator::Mul => {
                    let right = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    let left = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    stack.push(StackValue::Value(left * right));
                }
                Operator::Div => {
                    let right = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    let left = match stack.pop().unwrap() {
                        StackValue::Value(num) => num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(&id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    stack.push(StackValue::Value(left / right));
                }
                Operator::Print => {
                    let val = match stack.last().unwrap() {
                        StackValue::Value(num) => *num,
                        StackValue::Ident(id) => *state
                            .variables
                            .get(id)
                            .expect("TODO: handle variable that is undefined"),
                    };

                    print!(" {val}");
                }
            },
        }
    }
}
