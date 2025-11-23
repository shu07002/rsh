use std::fs::{File, OpenOptions};
use std::process::{Command, Stdio};

use crate::job;
use crate::parser::SimpleCommand;

pub fn execute_simple(cmd: &SimpleCommand, cmdline: &str) {
    let mut process = Command::new(&cmd.program);
    process.args(&cmd.args);

    if let Some(ref file) = cmd.redirect_in {
        match File::open(file) {
            Ok(f) => {
                process.stdin(Stdio::from(f));
            }
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
            }
            Err(e) => {
                eprintln!("failed to open output file {}: {}", file, e);
                return;
            }
        };
    }

    match process.spawn() {
        Ok(mut child) => {
            if cmd.background {
                println!("[bg] pid:{} {:?}", child.id(), cmd.program);
                job::add_single_job(child.id(), cmdline.to_string());
            } else {
                let _ = child.wait();
            }
        }
        Err(e) => {
            eprintln!("failed to execute {}: {}", cmd.program, e);
        }
    }
}

pub fn pipeline_exec(
    commands: &[SimpleCommand],
    cmdline: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
        } else if let Some(output) = previous {
            command.stdin(Stdio::from(output));
        }

        if i == commands.len() - 1 {
            if let Some(ref file) = cmd.redirect_out {
                let f = File::create(file)?;
                command.stdout(Stdio::from(f));
            } else if let Some(ref file) = cmd.append_out {
                let f = OpenOptions::new().append(true).create(true).open(file)?;
                command.stdout(Stdio::from(f));
            } else if background {
                // 백그라운드면 stdout을 부모 터미널로 보내지 않는다
                command.stdout(Stdio::null());
            } else {
                command.stdout(Stdio::inherit());
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

        let id = job::add_pipeline_job(pids, cmdline.to_string());
        println!("[bg] background pipeline job started, job id: {}", id);
        return Ok(());
    }

    for mut child in children {
        let _ = child.wait();
    }

    Ok(())
}
