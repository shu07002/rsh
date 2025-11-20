mod parser;
mod job;

use std::io::{self, Write};
use std::fs::{File, OpenOptions};
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

        if input == "jobs" {
            job::list_jobs();
            continue;
        }

        // kill %n
        if input.starts_with("kill %") {
            if let Some(idstr) = input.strip_prefix("kill %") {
                if let Ok(id) = idstr.parse::<usize>() {
                    job::kill_job(id);
                }
            }
            continue;
        }

        match parser::parse(input) {
            Ok(ParsedCommand::Pipeline(cmds)) => {
                if let Err(e) = pipeline_exec(&cmds, input) {
                    eprintln!("pipeline error: {}", e);
                }
            }
            Ok(ParsedCommand::Simple(cmd)) => execute_simple(&cmd, input),
            Err(ParseError::EmptyInput) => continue,
            Err(e) => {
                eprintln!("parse error: {}", e);
            }
        }
    }
}

fn execute_simple(cmd: &SimpleCommand, cmdline: &str) {
    let mut process = Command::new(&cmd.program);
    process.args(&cmd.args);

    if let Some(ref file) = cmd.redirect_in {
        match File::open(file) {
            Ok(f) => {
                process.stdin(Stdio::from(f));
            },
            Err(e) => {
                eprintln!("failed to open input file {}: {}", file, e);
                return;
            }
        }
    }

    if let Some(ref file) = cmd.redirect_out {
        match File::create(file) {
            Ok(f) => {
                process.stdout(Stdio::from(f));
            }
            Err(e) => {
                eprintln!("failed to open output file {}: {}", file, e);
                return;
            }
        };
    }

    if let Some(ref file) = cmd.append_out {
        match OpenOptions::new().append(true).create(true).open(file) {
            Ok(f) => {
                process.stdout(Stdio::from(f));
            },
            Err(e) => {
                eprintln!("failed to open output file {}: {}", file, e);
                return;
            }
        };
    }

    match process.spawn() {
        Ok(mut child) => {
            if  cmd.background {
                // 백그라운드 실행 시 바로 리턴
                println!("[bg] pid:{} {:?}", child.id(), cmd.program);
                job::add_single_job(child.id(), cmdline.to_string());
            } else {
                // 포그라운드 실행 시 완료 대기
                let _ = child.wait();
            }
        },
        Err(e) => {
            eprintln!("failed to execute {}: {}", cmd.program, e);
        }
    }
}

fn pipeline_exec(commands: &[SimpleCommand], cmdline: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut previous = None;
    let mut children = Vec::new();
    

    let background = commands.last().unwrap().background;

    for (i, cmd) in commands.iter().enumerate() {
        let mut command = Command::new(&cmd.program);
        command.args(&cmd.args);

        if i == 0 {
            if let Some(ref file) = cmd.redirect_in {
                let f = File::open(file)?;
                command.stdin(Stdio::from(f));
            } else if let Some(output) = previous {
                command.stdin(Stdio::from(output));
            }
        } else {
            if let Some(output) = previous {
                command.stdin(Stdio::from(output));
            }
        }

        // stdout 연결: 마지막 명령만 inherit
        if i == commands.len() - 1 {
            
            if let Some(ref file) = cmd.redirect_out {
                let f = File::create(file)?;
                command.stdout(Stdio::from(f));
            } else if let Some(ref file) = cmd.append_out {
                let f = OpenOptions::new().append(true).create(true).open(file)?;
                command.stdout(Stdio::from(f));
            } else {
                if background {
                    // 백그라운드면 stdout을 부모 터미널로 보내지 않는다
                    command.stdout(Stdio::null());
                } else {
                    command.stdout(Stdio::inherit());
                }
            }
        } else {
            command.stdout(Stdio::piped());
        }

        let mut child = command.spawn()?;
        
        previous = child.stdout.take();
        children.push(child);
    }

    if background {
        let mut pids = Vec::new();
        for child in &children {
            pids.push(child.id());
        }

        job::add_pipeline_job(pids, cmdline.to_string());
        println!("[bg] pipeline job started");
        return Ok(());
    }

    // 모든 child wait
    for mut child in children {
        let _ = child.wait();
    }

    Ok(())
}
