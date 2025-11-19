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

    pub redirect_in: Option<String>,
    pub redirect_out: Option<String>,
    pub append_out: Option<String>,
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
    let mut tokens= segment.split_whitespace().peekable();

    let program = match tokens.next() {
        Some(p) => p.to_string(),
        None => return Err(ParseError::EmptyCommand)
    };

    let mut args = Vec::new();
    let mut redirect_in = None;
    let mut redirect_out = None;
    let mut append_out = None;

    while let Some(tk) = tokens.next() {
        match tk {
            "<" => {
                let file = tokens.next().ok_or(ParseError::EmptyCommand)?;
                redirect_in = Some(file.to_string());
            }
            ">" => {
                let file = tokens.next().ok_or(ParseError::EmptyCommand)?;
                redirect_out = Some(file.to_string());
            }
            ">>" => {
                let file = tokens.next().ok_or(ParseError::EmptyCommand)?;
                append_out = Some(file.to_string());
            }
            _=> {
                args.push(tk.to_string());
            }
        }
    }


     Ok(SimpleCommand {
        program,
        args,
        redirect_in,
        redirect_out,
        append_out,
    })
}
