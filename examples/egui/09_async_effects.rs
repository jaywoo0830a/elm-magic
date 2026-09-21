//! egui 예제 09 — 비동기 효과 `<-` 와 낙관적 업데이트
//!
//! **언제 쓰나**: 클릭/마운트 뒤에 네트워크·파일·계산 결과를 상태로 받아올 때.
//!
//! **규칙**
//! - `slot <- f(args)` — `f`는 `async fn(f_args) -> slot 타입`. 결과가 슬롯에 들어간다.
//! - 여러 타깃을 한 번에: `status, users <- load_users()` (튜플 반환 순서대로).
//! - `on_mount { … }` — 인스턴스가 화면에 붙을 때 한 번.
//! - `slot <- f() after 300ms` — 지연 효과(예제 12).
//! - 효과는 **`flush()`/`advance()` 전에는 실행되지 않는다** — 그래서 테스트가
//!   결정적이다(`tests/callbacks.rs`가 계약).
//!
//! **베스트 패턴 — 낙관적 업데이트**: `count += 1;`을 **먼저** 하고 `<-`를 나중에
//! 둔다. UI는 즉시 반응하고, 응답이 오면 서버 값으로 덮어쓴다. 실패 시 되돌리는
//! 것은 `<-` 결과에 실패 상태를 담아 표현한다(예제 19의 3분기).
//!
//! **주의**: 효과 함수 이름은 `mock!`/`mock_stream!`의 **키**가 된다 — 같은 이름을
//! 여러 곳에서 쓰면 목도 함께 적용된다.

use elm_magic::prelude::*;

async fn load_users() -> (String, Vec<String>) {
    ("done".to_string(), vec!["alice".to_string(), "bob".to_string()])
}

async fn optimistic_like(n: i32) -> i32 {
    n * 10 // 서버가 돌려주는 "진짜" 값이라고 하자
}

elm_magic::view! {
    fn Users(users: Vec<String> = vec![], status = String::from("idle")) {
        on_mount { status, users <- load_users() }
        <Col>
            <Button on_click={status, users <- load_users()}>"reload"</Button>
            "status: {status}"
            <For each={users} as={u}>
                <Row>"{u}"</Row>
            </For>
        </Col>
    }
}

elm_magic::view! {
    fn Likes(count = 0) {
        <Col>
            <Button on_click={
                count += 1;                  // ① 즉시 반영 (낙관적)
                count <- optimistic_like(count) // ② 응답이 오면 덮어씀
            }>"like"</Button>
            "likes: {count}"
        </Col>
    }
}

/// ── 실제 앱에서 효과를 구동한다 (중요) ──────────────────────
///
/// **어댑터는 효과를 실행하지 않는다.** `frame`/`render`는 트리만 만들고, `<-`가
/// 예약한 효과는 아레나 큐에 쌓일 뿐이다. 테스트에서는 `app.flush()`/`app.advance()`가
/// 대신 구동하지만, 앱 모드에서는 **앱이 프레임마다 한 번** 아래를 불러야 한다.
/// (`Arena::take_due`/`set_now`는 공개 API다 — `src/state.rs`)
fn drive_effects(ctx: &mut Ctx, now_ms: u64) -> bool {
    ctx.arena.set_now(now_ms);
    let mut ran = false;
    loop {
        let due = ctx.arena.take_due();
        if due.is_empty() {
            break;
        }
        for effect in due {
            effect(&mut ctx.arena);
        }
        ran = true;
    }
    ran
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_effect_fills_state() {
        let mut app = elm_magic::mount!(Users);
        app.expect_text("status: idle");   // flush 전
        app.flush();
        app.expect_text("status: done");
        app.expect_text("alice");
    }

    #[test]
    fn optimistic_then_server_value() {
        let mut app = elm_magic::mount!(Likes);
        app.click("like");
        app.expect_text("likes: 1");       // flush 전 = 낙관적 값
        app.flush();
        app.expect_text("likes: 10");      // 서버 값
    }
}

