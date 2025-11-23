use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use crate::job;
use crate::parser::SimpleCommand;

const CHANNEL_DEPTH: usize = 16; 

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
    if commands.is_empty() {
        return Ok(());
    }

    let background = commands.last().unwrap().background;
    let mut prev_receiver: Option<Receiver<Vec<u8>>> = None;
    let mut children = Vec::new();
    let mut io_threads = Vec::new();

    for (i, cmd) in commands.iter().enumerate() {
        let is_last = i == commands.len() - 1;
        let (output_tx, next_rx) = if is_last {
            (None, None)
        } else {
            let (tx, rx) = mpsc::sync_channel(CHANNEL_DEPTH);
            (Some(tx), Some(rx))
        };

        let (child, mut threads) =
            spawn_stage(cmd, prev_receiver.take(), output_tx, background, is_last)?;
        children.push(child);
        io_threads.append(&mut threads);
        prev_receiver = next_rx;
    }

    if background {
        let pids: Vec<u32> = children.iter().map(|c| c.id()).collect();
        let id = job::add_pipeline_job(pids, cmdline.to_string());
        println!("[bg] background pipeline job started, job id: {}", id);
        // detach: let threads run; do not wait
        return Ok(());
    }

    for mut child in children {
        let _ = child.wait();
    }

    for handle in io_threads {
        let _ = handle.join();
    }

    Ok(())
}

fn spawn_stage(
    cmd: &SimpleCommand,
    input_rx: Option<Receiver<Vec<u8>>>,
    output_tx: Option<SyncSender<Vec<u8>>>,
    background: bool,
    is_last: bool,
) -> Result<(std::process::Child, Vec<thread::JoinHandle<()>>), Box<dyn std::error::Error>> {
    let mut command = Command::new(&cmd.program);
    command.args(&cmd.args);

    if let Some(ref file) = cmd.redirect_in {
        let f = File::open(file)?;
        command.stdin(Stdio::from(f));
    } else if input_rx.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::inherit());
    }

    if is_last {
        if let Some(ref file) = cmd.redirect_out {
            let f = File::create(file)?;
            command.stdout(Stdio::from(f));
        } else if let Some(ref file) = cmd.append_out {
            let f = OpenOptions::new().append(true).create(true).open(file)?;
            command.stdout(Stdio::from(f));
        } else if background {
            command.stdout(Stdio::null());
        } else {
            command.stdout(Stdio::inherit());
        }
    } else {
        command.stdout(Stdio::piped());
    }

    let mut child = command.spawn()?;
    let mut handles = Vec::new();

    if let Some(rx) = input_rx {
        if let Some(stdin) = child.stdin.take() {
            handles.push(thread::spawn(move || {
                let mut stdin = stdin;
                for chunk in rx {
                    if stdin.write_all(&chunk).is_err() {
                        break;
                    }
                }
            }));
        }
    }

    if !is_last {
        if let (Some(tx), Some(stdout)) = (output_tx, child.stdout.take()) {
            handles.push(thread::spawn(move || {
                let mut stdout = stdout;
                let mut buf = [0u8; 8192];
                loop {
                    match stdout.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }));
        }
    }

    Ok((child, handles))
}
