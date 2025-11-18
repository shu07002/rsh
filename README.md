# rsh – Rust로 구현한 Shell  
**Process 기반 실행 vs Thread 기반 실행 성능 비교 프로젝트**

## 📌 개요
`rsh`는 Rust로 구현한 Linux Shell입니다.  
Bash와 유사한 기본 기능을 제공하며, 동일한 명령을 두 가지 실행 모델(process / thread)로 수행해 내부 실행 방식에 따른 성능 차이를 비교하는 것을 목적으로 합니다.

### 비교 대상 실행 모델
- **process mode**  
  - Bash와 유사한 구조  
  - 각 명령을 OS 프로세스로 실행 (`Command::new`)  
  - OS 파이프(`Stdio::piped`) 기반 처리  

- **thread mode**  
  - Rust 스레드 기반 실행 모델  
  - 파이프는 channel 기반 스트림으로 처리  
  - 컨텍스트 스위칭 비용이 적고 병렬성이 높음  

쉘 내부에서 실행 모드를 전환할 수 있습니다:
```
rsh> mode process
rsh> mode thread
```

---

## 📚 제공 기능 (Features)

### ✔ 기본 명령 실행
```
rsh> ls -al
```

### ✔ 파이프라인
```
rsh> ls | grep src | wc -l
```

### ✔ 리다이렉션
```
rsh> echo hi > out.txt
```

### ✔ 백그라운드 실행 (&)
```
rsh> sleep 3 &
```

### ✔ job 관리
```
rsh> jobs
```

### ✔ 내부 명령어
- cd  
- exit  
- help  
- jobs  
- kill  
- mode (process/thread)

---

## 🏛 아키텍처 구조

```
rsh/
├── main.rs            # REPL, 모드 관리
├── parser.rs          # 파이프/리다이렉션/& 파싱
├── builtins.rs        # cd, exit, mode 등 내장 명령어
├── executor/
│   ├── process.rs     # 프로세스 기반 실행 엔진
│   └── thread.rs      # 스레드 기반 실행 엔진
├── job.rs             # 백그라운드 job 관리
└── util.rs            # 공용 유틸리티
```

---

## 🛠 기능 구현 로드맵 (WBS)

### 1단계 — Shell 기본 골격
- REPL 루프  
- 사용자 입력 처리  
- 공백 단위 토큰 파싱  

### 2단계 — 기본 명령 실행
- 단일 명령 실행  
- Command::new 기반 프로세스 생성  

### 3단계 — 파이프라인 구현
- `|` 파싱  
- process mode: `Stdio::piped()` 연결  
- thread mode: channel 기반 스트림 처리  

### 4단계 — 리다이렉션 구현
- `>` 처리  
- stdout redirect  

### 5단계 — 백그라운드 실행(&) + job manager
- bg 태스크 등록  
- job 리스트 관리  

### 6단계 — 내장 명령어 구현
- cd, exit, help  
- jobs, kill, mode  

### 7단계 — process mode 완성
- 파이프/리다이렉션 통합  
- 종료 코드 처리  
- 여러 명령 체인 처리  

### 8단계 — thread mode 완성
- 스레드 기반 파이프 스트림  
- reader/writer 구조  
- join handle 관리  