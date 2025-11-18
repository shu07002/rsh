let x = 5;
x = 10;   // ❌ 오류 발생

let mut x = 5;
x = 10;   // ⭕ 허용됨

let       → 값 못 바꿈
let mut   → 값 바꿀 수 있음

&x      = 이 객체를 읽어볼 수만 있는 열쇠
&mut x  = 이 객체를 읽고 *수정까지* 가능한 열쇠

C에서의 포인터랑 비슷하다
--------------------------------------------------------------
*unwrap*은 Result나 Option에서 성공값을 꺼내는 방법임

Some = Option
Ok = Result

Option<T>: 값이 있을 수도 있고 없을 수도 있음
- Some(값) : 있을 때
- None : 없을 때

Result<T,E>: 성공하거나 실패할 수 있음
Ok(성공값)
Err(에러정보)

러스트에는 `널`이랑 `예외`가 없음
- 그래서 값이 없을 수도 있을 때는 **Option**
- 실패할 수 있을 때는 **Result**

parts.next()는 Option을 반환함
- Some("ls")
- Some("ehco")
- None


```rust
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
        // 공백 기준으로 명령어들 파싱해서 이터레이터 만들고
        let cmd = parts.next().unwrap();
        // 이터레이터의 첫번째 항목을 꺼냄
```
이미 앞에서 공백에 대한 예외를 처리했음.
그래서 unwrap()을 써도 안전함
--------------------------------------------------------------
```rust
let args: Vec<&str> = parts.collect();
```

여기서 `Vec<&str>`의 `&str`인 이유는??

일단 str은 문자열 조각의 타입
말 그대로 문자열을 저장할 수 있지만 사이즈가 정해져있지 않아서 변수로 저장할 수 없고 Vec같은 컨테이너에도 못 넣음

반면에 &str은 참조이기 때문에 크기가 고정된다
```rust
&str = 문자열 시작 주소 + 길이 정보
```

str vs String
str은 걍 찐빼이 문자열 조각들
힙이나 스택 어딘가에 놓여있는 순수한 문자열 데이터들임
근데 사이즈는 알 수 없음

대신 &str은?
```rust
struct &str {
    ptr: *const u8,
    len: usize
}
```

```rust
pub struct String {
    ptr: *mut u8,   // 힙에 있는 문자열 데이터의 시작주소
    len: usize,     // 문자열 길이
    capacity: usize // 힙에 예약된 공간 크기
}
```



parts의 원본은 애초에 input임
만약에 여기서 그냥 str로 작성해줬다면 원본을 참조하는게 아니라 원본을 복사한 걸 전달하는 셈임. 그래서 원본의 참조 주소를 전달.
```rust
input:  |l|s| |-|a|l| |/|h|o|m|e|
         ↑     ↑       ↑
         "ls" "-al"    "/home"

&input[0..2]
&input[3..6]
&input[7..12]
```

그리고 String이 아닌 이유는??
Vec<String>을 사용하면 
- 원래 문자열에서 조각을 하나씩 복사해서 새로운 스트링을 만들고
- 메모리 재할당 하고 
- 힙 메모리 잡고

결론적으로 느려지고 비효율적이다.
--------------------------------------------------------------
*spawn()*는 뭐냐

스폰은 새 프로세스를 생성하고 바로 실행한 뒤 실행된 프로세스를 나타내는 child 핸들을 반환한다.

리눅스 내부적으로는 `fork()`, `execve()` 이 두개를 조합한 것과 똑같이 동작한다.

그래서 내가 만드는 프로세스가 fork()해서 자식 프로세스 생성하고
execve()로 ls든 grep이든 cat이든 실행한다.

스폰은 성공/실패를 나타내는 Result를 반환한다.
Result<Child, Erorr>

만약에 명령이 존재하지 않거나 실행권한이 없거나 PATH에 없는 프로그램을 실행하려고 하면 실패를 반환할거다.

그럼 *match*는 뭐냐
```rust
match 값 {
    패턴1 => 실행문1,
    패턴2 => 실행문2,
    _ => 기본실행문,
}
```
매치는 값이 어떤 패턴과 맞는기 검사해서 맞는 블록을 실행함
여기서 값이 아니라 `패턴 매칭`이라는걸 생각하는게 좋음

얼핏보면 switch-case문이랑 비슷한데 그건 값만 비교하는거소 이건 패턴임!

그리고 여러개의 패턴이랑 매칭되면 가장 먼저 일치된 패턴 하나만 실행하고 땡

