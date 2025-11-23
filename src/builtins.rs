use std::env;
use std::path::Path;
use crate::job;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    Process,
    Thread,
}

pub fn handle_builtin(input: &str, mode: &mut ExecMode) -> bool {
    let mut parts = input.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };

    match cmd {
        "exit" => {
            std::process::exit(0);
        }
        "help" => {
            println!("builtins: cd, exit, help, jobs, kill %id, mode [process|thread]");
            true
        }
        "cd" => {
            let target = parts
                .next()
                .map(|s| s.to_string())
                .or_else(|| env::var("HOME").ok())
                .unwrap_or_else(|| "/".to_string());

            if let Err(e) = env::set_current_dir(Path::new(&target)) {
                eprintln!("cd: {}: {}", target, e);
            }
            true
        }
        "jobs" => {
            job::list_jobs();
            true
        }
        "kill" => {
            if let Some(arg) = parts.next() {
                if let Some(id) = arg.strip_prefix('%').and_then(|s| s.parse::<usize>().ok()) {
                    job::kill_job(id);
                } else {
                    eprintln!("kill: usage: kill %<jobid>");
                }
            } else {
                eprintln!("kill: usage: kill %<jobid>");
            }
            true
        }
        "mode" => {
            match parts.next() {
                Some("process") => {
                    *mode = ExecMode::Process;
                    println!("switched to process mode");
                }
                Some("thread") => {
                    *mode = ExecMode::Thread;
                    println!("switched to thread mode");
                }
                _ => {
                    println!("current mode: {:?}", mode);
                    println!("usage: mode process | mode thread");
                }
            }
            true
        }
        _ => false,
    }
}
