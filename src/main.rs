mod parser;
mod job;
mod builtins;
mod executor;

use std::io::{self, Write};
use builtins::{ExecMode, handle_builtin};
use executor::{process, thread};

use parser::{Command as ParsedCommand, ParseError};


fn main() {
    let mut mode = ExecMode::Process;
    loop {
        print!("rsh> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("input error");
            continue;
        }


        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if handle_builtin(input, &mut mode) {
            continue;
        }

        match parser::parse(input) {
            Ok(ParsedCommand::Pipeline(cmds)) => {
                let result = match mode {
                    ExecMode::Process => process::pipeline_exec(&cmds, input),
                    ExecMode::Thread => thread::pipeline_exec(&cmds, input),
                };

                if let Err(e) = result {
                    eprintln!("pipeline error: {}", e);
                }
            }
            Ok(ParsedCommand::Simple(cmd)) => match mode {
                ExecMode::Process => process::execute_simple(&cmd, input),
                ExecMode::Thread => thread::execute_simple(&cmd, input),
            },
            Err(ParseError::EmptyInput) => continue,
            Err(e) => {
                eprintln!("parse error: {}", e);
            }
        }
    }
}
