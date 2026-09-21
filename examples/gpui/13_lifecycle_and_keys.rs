//! gpui 예제 13 — 수명/단축키/시계를 gpui에서 구동하기
//!
//! **언제 쓰나**: 붙을 때 로드, 떨어질 때 정리, 전역 단축키, 주기 갱신.
//!
//! **elm 쪽 문법** (플랫폼과 무관, `tests/lifecycle.rs`)
//! - `on_mount { .. }` / `on_unmount { .. }` — 인스턴스 수명에 한 번씩.
//!   unmount 시점의 **지역 슬롯 쓰기는 버려진다** → 전역(`#[store]`)만 갱신한다.
//! - `on_key("Ctrl+S") { .. }` — 핸들러는 `Ctx::keys`에 (키, 핸들러)로 쌓인다.
//! - `on_tick(500ms) { .. }` / `<- f() after 300ms` — 시계가 있어야 due가 된다.
//!
//! **gpui에서 누가 구동하나 (중요)**
//! - **단축키**: 어댑터는 키를 전역으로 훑지 않는다. 앱이 gpui 키 바인딩/`on_key_down`에서
//!   `ctx.keys`의 핸들러를 찾아 호출해야 한다(아래 `dispatch_key`). 테스트의
//!   `app.press_key("Ctrl+S")`가 하는 일이 정확히 이것이다.
//! - **시계**: `arena.set_now(ms)`로 앱이 밀어 준다. `on_tick`은 다음 due까지 기다린다
//!   (놓친 틱을 몰아서 오지 않는다).
//! - **unmount**: `frame`이 사라진 인스턴스를 감지해 `on_unmount`를 큐에 넣지만,
//!   실행은 **앱이 `take_pending()`으로** 한다(효과와 같은 원리).
//!
//! **베스트 패턴**: 앱 루프에 `tick()` 하나를 두고 (1) 시계 전진 → (2) due 효과 →
//! (3) 스트림 펌프 → (4) 변경 시 `cx.notify()`를 순서대로 한다(예제 20의 골격).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

async fn api_user(id: i32) -> String {
    format!("user-{id}")
}

elm_magic::view! {
    fn Editor(text = String::new()) {
        on_key("Ctrl+S") { text = String::from("saved") }
        on_key("Ctrl+Z") { text = String::from("undone") }
        <Text>"{text}"</Text>
    }
}

elm_magic::view! {
    fn Clock(now = 0) {
        on_tick(500ms) { now += 1 }
        <Text>"tick: {now}"</Text>
    }
}

elm_magic::view! {
    fn Profile(id = 1, user = String::new()) {
        on_mount { user <- api_user(id) }
        <Text>"user: {user}"</Text>
    }
}

/// 앱의 키 디스패치 — `Ctx::keys`는 `on_key`가 마지막 렌더에 등록한 목록이다.
fn dispatch_key(ctx: &mut Ctx, key: &str) -> bool {
    let handler = ctx
        .keys
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, h)| h.clone());
    match handler {
        Some(h) => {
            h(&mut ctx.arena);
            true
        }
        None => false,
    }
}

/// 앱 루프의 나머지 절반: 시계 전진 + due 효과 실행.
fn tick(ctx: &mut Ctx, now_ms: u64) -> bool {
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
        div().child(cx.new(ElmView::<Clock>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_dispatch() {
        let mut app = elm_magic::mount!(Editor);
        app.press_key("Ctrl+S");
        app.expect_text("saved");
        app.press_key("Ctrl+Z");
        app.expect_text("undone");
    }

    #[test]
    fn tick_respects_the_period() {
        let mut app = elm_magic::mount!(Clock);
        app.advance(500);
        app.assert_text("tick: 1");
        app.advance(200); // 주기가 덜 지났다
        app.assert_text("tick: 1");
        app.advance(300);
        app.assert_text("tick: 2");
    }
}