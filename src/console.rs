use std::io::{Write, stdin, stdout};

use parsr::{input::StrView, interner::Interner};

use crate::{
    raw_token::parse_raw_tokens,
    run::{State, run},
    tokens::resolved_tokens,
};

pub fn console() {
    let mut interner = Interner::new();
    let mut state = State::new();

    loop {
        print!("> ");

        stdout().flush().unwrap();

        let mut line = String::new();

        let Ok(_) = stdin().read_line(&mut line) else {
            continue;
        };

        if line.starts_with(":q") {
            break;
        }

        let mut view = StrView::new(&line);

        let raw_tokens = parse_raw_tokens(&mut view, &mut interner)
            .unwrap()
            .map(|r| r.expect("TODO: handle error"));

        let tokens = resolved_tokens(raw_tokens).map(|r| r.expect("TODO: handle error").inner);

        run(&mut state, tokens);

        println!();
    }
}
