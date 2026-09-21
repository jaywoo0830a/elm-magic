//! gpui 예제 11 — 비동기 효과 `<-` 와 **누가 효과를 실행하는가**
//!
//! **언제 쓰나**: 클릭/마운트 뒤에 네트워크 결과를 상태로 받아올 때.
//!
//! **가장 중요한 사실**: `<-`가 예약한 효과는 **어댑터가 실행하지 않는다**.
//! `frame`/`render`는 트리만 만들고, 효과는 아레나 큐(`take_due`)에 쌓인다.
//! 테스트에서는 `app.flush()`/`app.advance()`가 구동하지만, **실제 앱에서는 앱이
//! 프레임마다 `take_due()`를 돌려야** 한다(아래 `drive_effects`).
//! `PendingEffect = Box<dyn FnOnce(&mut Arena)>`이고 `Arena`는 공개 API다.
//!
//! **규칙**
//! - `slot <- f(args)` — `f`는 `async fn(f_args) -> slot 타입`.
//! - 여러 타깃: `status, users <- load_users()` (튜플 반환 순서대로).
//! - `on_mount { .. }` — 인스턴스가 붙을 때 한 번.
//! - `slot <- f() after 300ms` — 지연. `arena.set_now(ms)`로 시계를 전진시켜야 due가 된다.
//!
//! **낙관적 업데이트**: `count += 1;`을 **먼저**, `<-`를 나중에 — UI가 즉시 반응하고
//! 응답이 오면 덮어쓴다.
//!
//! **베스트 패턴**
//! - 앱 루프에서 `drive_effects(&mut view.ctx_mut(), now_ms)`를 한 번 부른 뒤
//!   바뀌었으면 `cx.notify()`한다. 시계는 `std::time::Instant`로 앱이 관리한다.
//! - 효과 함수 이름은 `mock!`의 키다 — 같은 이름을 여러 곳에서 쓰면 목도 함께 적용된다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

async fn load_users() -> (String, Vec<String>) {
    ("done".to_string(), vec!["alice".to_string(), "bob".to_string()])
}

async fn optimistic_like(n: i32) -> i32 {
    n * 10
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
                count += 1;                     // ① 즉시 반영
                count <- optimistic_like(count)  // ② 응답이 오면 덮어씀
            }>"like"</Button>
            "likes: {count}"
        </Col>
    }
}

/// 앱 루프가 프레임마다 부른다 — due 효과를 실행하고, 하나라도 돌았으면 `true`.
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

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<Likes>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_effect_fills_state() {
        let mut app = elm_magic::mount!(Users);
        app.expect_text("status: idle"); // flush 전
        app.flush();
        app.expect_text("status: done");
        app.expect_text("alice");
    }

    #[test]
    fn optimistic_then_server_value() {
        let mut app = elm_magic::mount!(Likes);
        app.click("like");
        app.expect_text("likes: 1");
        app.flush();
        app.expect_text("likes: 10");
    }
}