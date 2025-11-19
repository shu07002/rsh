use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Command {
    Simple(SimpleCommand),
    Pipeline(Vec<SimpleCommand>),
}

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    EmptyCommand,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "no input provided"),
            ParseError::EmptyCommand => write!(f, "command segment is empty"),
        }
    }
}

impl Error for ParseError {}

pub fn parse(input: &str) -> Result<Command, ParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let segments: Vec<&str> = trimmed
        .split('|')
        .map(|segment| segment.trim())
        .collect();

    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(ParseError::EmptyCommand);
    }

    let mut commands = Vec::with_capacity(segments.len());
    for segment in segments {
        commands.push(parse_simple(segment)?);
    }

    if commands.len() == 1 {
        Ok(Command::Simple(commands.remove(0)))
    } else {
        Ok(Command::Pipeline(commands))
    }
}

fn parse_simple(segment: &str) -> Result<SimpleCommand, ParseError> {
    let mut parts = segment.split_whitespace();
    let program = parts.next().ok_or(ParseError::EmptyCommand)?;
    let args = parts.map(|arg| arg.to_string()).collect();

    Ok(SimpleCommand {
        program: program.to_string(),
        args,
    })
}
