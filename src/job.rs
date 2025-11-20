use std::sync::{Mutex, OnceLock};
use nix::unistd::Pid;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

#[derive(Debug, Clone)]
pub enum JobState {
    Running,
    Done,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,
    pub pids: Vec<u32>,   // pipeline 전체 PID
    pub cmdline: String,  // jobs 출력용 원본 문자열
    pub state: JobState,
}

static JOB_TABLE: OnceLock<Mutex<Vec<Job>>> = OnceLock::new();

fn table<'a>() -> &'a Mutex<Vec<Job>> {
    JOB_TABLE.get_or_init(|| Mutex::new(Vec::new()))
}

static NEXT_ID: OnceLock<Mutex<usize>> = OnceLock::new();

fn next_id() -> usize {
    let m = NEXT_ID.get_or_init(|| Mutex::new(1));
    let mut guard = m.lock().unwrap();
    let id = *guard;
    *guard += 1;
    id
}

/// pipeline job 등록
pub fn add_pipeline_job(pids: Vec<u32>, cmdline: String) {
    let job = Job {
        id: next_id(),
        pids,
        cmdline,
        state: JobState::Running,
    };

    let mut jobs = table().lock().unwrap();
    jobs.push(job);
}

/// 단일 프로세스 job 등록
pub fn add_single_job(pid: u32, cmdline: String) {
    add_pipeline_job(vec![pid], cmdline);
}

/// jobs 명령 출력
pub fn list_jobs() {
    clean_jobs(); // dead job 자동 정리

    let jobs = table().lock().unwrap();
    for job in jobs.iter() {
        let state_str = match job.state {
            JobState::Running => "Running",
            JobState::Done => "Done",
        };

        println!("[{}] {}   {}", job.id, state_str, job.cmdline);
    }
}

/// 죽은 job 자동 제거 (bash의 clean-up 역할)
pub fn clean_jobs() {
    let mut jobs = table().lock().unwrap();

    jobs.retain_mut(|job| {
        let mut alive = false;

        for pid in &job.pids {
            let res = waitpid(Pid::from_raw(*pid as i32), Some(WaitPidFlag::WNOHANG));
            match res {
                Ok(WaitStatus::StillAlive) => alive = true,
                Ok(_) => {}
                Err(_) => {}
            }
        }

        if alive {
            job.state = JobState::Running;
            true
        } else {
            job.state = JobState::Done;
            // Done job은 jobs 출력 후 사라져야 함 (bash도 마찬가지)
            false
        }
    });
}

/// kill %id
pub fn kill_job(id: usize) {
    let jobs = table().lock().unwrap();

    if let Some(job) = jobs.iter().find(|j| j.id == id) {
        for pid in &job.pids {
            let _ = kill(Pid::from_raw(*pid as i32), Signal::SIGKILL);
        }
        println!("Killed job [{}]", id);
    } else {
        eprintln!("no such job: {}", id);
    }
}
