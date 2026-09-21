//! egui 예제 12 — 수명/입력/시계: `on_mount` / `on_unmount` / `on_key` / `on_tick`
//!
//! **언제 쓰나**: 화면이 붙고 떨어지는 순간, 단축키, 주기 갱신.
//!
//! **도구와 계약** (`tests/lifecycle.rs`, `tests/store.rs`)
//! - `on_mount { .. }` — 인스턴스가 **처음 화면에 붙을 때 한 번**.
//! - `on_unmount { .. }` — 사라질 때 한 번. **이 시점의 지역 슬롯 쓰기는 버려진다**
//!   (0.7.3) — 전역(`#[store]`) 상태만 갱신하는 데 쓴다.
//! - `on_key("Ctrl+S") { .. }` — 단축키. 문자열은 `app.press_key("Ctrl+S")`로 검증한다.
//! - `on_tick(500ms) { .. }` — 주기. **놓친 틱은 몰아서 오지 않는다**(다음 주기부터).
//! - 지연 효과 `slot <- f() after 300ms` / `1min` — 예약된 일회성.
//!
//! **베스트 패턴**
//! - "붙을 때 로드"는 `on_mount` + `<-` 한 줄이면 된다(예제 09).
//! - 정리 작업은 지역 상태가 아니라 **전역 상태/외부 자원**에 한다 — 지역 슬롯은
//!   unmount 시점에 이미 죽어 있다.
//! - `on_tick`은 "지금 시각" 슬롯 하나만 갱신하고, 표시 형식은 파생 값으로 만든다.

use elm_magic::prelude::*;

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
    fn Profile(id = 1, user = String::new(), fresh = String::new()) {
        on_mount {
            if user.is_empty() { user <- api_user(id) }
            fresh <- api_user(id) after 300ms
        }
        <Col>
            "user: {user}"
            "fresh: {fresh}"
        </Col>
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
    fn tick_fires_per_period() {
        let mut app = elm_magic::mount!(Clock);
        app.advance(500);
        app.assert_text("tick: 1");
        app.advance(200); // 주기가 덜 지났다
        app.assert_text("tick: 1");
        app.advance(300);
        app.assert_text("tick: 2");
    }

    #[test]
    fn delayed_effect_waits_for_the_clock() {
        let mut app = elm_magic::mount!(Profile);
        app.flush(); // 즉시 효과만
        app.assert_text("user: user-1");
        assert!(app.has_pending_after(), "300ms 효과가 예약돼 있어야 한다");
        app.advance(300);
        app.assert_text("fresh: user-1");
    }
}
