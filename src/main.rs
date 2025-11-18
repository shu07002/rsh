use std::io::{self, Write};
use std::process::Command;

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

        // 명령 파싱
        let mut parts = input.split_whitespace();
        // 공백 기준으로 명령어들 파싱해서 이터레이터 반환
        let cmd = parts.next().unwrap();
        // 이터레이터의 첫번째 항목을 꺼냄

        let args: Vec<&str> = parts.collect();
        // 나머지 항목들을 벡터로 수집
        // 파츠들은 이터레이터 이기 때문에 collect()로 벡터로 변환 가능

        // Linux: 실행 가능한 명령만 찾으면 됨
        let result = Command::new(cmd)
            .args(&args)
            .spawn();

        match result {
            Ok(mut child) => {
                // 외부 프로그램 종료 대기
                child.wait().unwrap();
            }
            Err(e) => {
                eprintln!("명령 실행 실패: {}", e);
            }
        }
    }
}
