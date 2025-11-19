mod parser;

use std::io::{self, Write};
use std::process::{Command, Stdio};

use parser::{Command as ParsedCommand, ParseError, SimpleCommand};

// io: 입출력 표준 라이브러리
// Write: flush() 처럼 출력 조작을 위해서 사용
// Command: 외부 프로그램 실행을 하기 위해 필요한 API (fork, exec,...)


fn main() {
    loop {
        // 프롬프트 출력
        print!("rsh> ");
        io::stdout().flush().unwrap();
        // 여기서 플러쉬가 없으면 버퍼에 남아있다가 다음 입력 시점에 출력됨
        // 그래서 플러쉬를 해줘야 바로 출력됨

        // 사용자 입력
        let mut input = String::new();
        //  입력 저장할 문자열
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("input error");
            continue;
        }
        // read_line에서 사용자가 엔터 칠 때까지 입력 받음 맨 끝에 개행 문자 포함됨


        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // exit 처리
        if input == "exit" {
            break;
        }

        match parser::parse(input) {
            Ok(ParsedCommand::Pipeline(cmds)) => {
                if let Err(e) = pipeline_exec(&cmds) {
                    eprintln!("pipeline error: {}", e);
                }
            }
            Ok(ParsedCommand::Simple(cmd)) => execute_simple(&cmd),
            Err(ParseError::EmptyInput) => continue,
            Err(e) => {
                eprintln!("parse error: {}", e);
            }
        }
    }
}

fn execute_simple(cmd: &SimpleCommand) {
    let result = Command::new(&cmd.program).args(&cmd.args).spawn();

    match result {
        Ok(mut child) => {
            // 외부 프로그램 종료 대기
            child.wait().unwrap();
        }
        Err(e) => {
            eprintln!("failed executing command: {}", e);
        }
    }
}

fn pipeline_exec(commands: &[SimpleCommand]) -> Result<(), Box<dyn std::error::Error>> {
    let mut previous = None;
    let mut children = Vec::new();

    for (i, cmd) in commands.iter().enumerate() {
        let mut command = Command::new(&cmd.program);
        command.args(&cmd.args);

        // stdin 연결
        if let Some(output) = previous {
            command.stdin(Stdio::from(output));
        }

        // stdout 연결: 마지막 명령만 inherit
        if i == commands.len() - 1 {
            command.stdout(Stdio::inherit());
        } else {
            command.stdout(Stdio::piped());
        }

        let mut child = command.spawn()?;
        previous = child.stdout.take();
        children.push(child);
    }

    // 모든 child wait
    for mut child in children {
        let _ = child.wait();
    }

    Ok(())
}
