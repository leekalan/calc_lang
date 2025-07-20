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
        print!("< ");

        stdout().flush().unwrap();

        let mut line = String::new();

        let Ok(_) = stdin().read_line(&mut line) else {
            continue;
        };

        if line.starts_with(":q") {
            break;
        }

        print!("> ");
        stdout().flush().unwrap();

        let mut view = StrView::new(&line);

        let raw_tokens = parse_raw_tokens(&mut view, &mut interner).unwrap();

        let tokens = resolved_tokens(raw_tokens);

        if let Err(err) = run(&mut state, tokens) {
            print!("\n\n!> {line}!> ");
            print!("{: <1$}", "", err.span.start);
            println!("{:~<1$}", "", err.span.end - err.span.start);
            print!("!> ");
            print!("{: <1$}", "", err.span.start);
            println!("^ ERROR: {}", err.inner);
        }

        println!();
    }
}
