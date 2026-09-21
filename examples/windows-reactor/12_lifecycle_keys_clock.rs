//! windows-reactor 예제 12 — 수명/단축키/시계를 Reactor에서 구동하기
//!
//! **언제 쓰나**: 붙을 때 로드, 떨어질 때 정리, 단축키, 주기 갱신.
//!
//! **elm 쪽 문법** (플랫폼과 무관, `tests/lifecycle.rs`·`tests/timers.rs`)
//! - `on_mount { .. }` / `on_unmount { .. }` — 인스턴스 수명에 한 번씩.
//!   unmount 시점의 **지역 슬롯 쓰기는 버려진다** → 전역(`#[store]`)만 갱신한다.
//! - `on_key("Ctrl+S") { .. }` — 핸들러는 `Ctx::keys`에 (키, 핸들러)로 쌓인다.
//! - `on_tick(500ms) { .. }` / `<- f() after 300ms` — **시계**가 있어야 due가 된다.
//!
//! **Reactor에서 누가 구동하나**
//! - **효과/스트림**: `ElmView::update`가 `drive::run`을 부른다(예제 09, 10).
//! - **단축키**: 어댑터는 키를 훑지 않는다. 호스트가 키 이벤트를 받아
//!   `drive::dispatch_key(ctx, "Ctrl+S")`를 부르거나, `KeyboardAccelerator`
//!   (`KeyAccelerators`)를 붙인 WinUI 컨트롤로 받아 같은 일을 한다.
//!   테스트의 `app.press_key("Ctrl+S")`가 하는 일이 정확히 `dispatch_key`다.
//! - **시계**: `drive::run_at(ctx, elapsed_ms)`로 밀어 준다. Reactor 0.100.0에는
//!   타이머 API가 없으므로 호스트가 시계를 공급한다(`std::time::Instant`).
//!
//! **베스트 패턴**: 호스트에 `tick` 하나를 두고 (1) 시계 전진 → (2) due 효과 →
//! (3) 스트림 펌프를 순서대로 한다(예제 20의 골격).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{drive, ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

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

async fn api_user(id: i32) -> String {
    format!("user-{id}")
}

struct Root {
    started: std::time::Instant,
}

impl Component for Root {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            started: std::time::Instant::now(),
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 수명/단축키/시계");
        let _ = self.started; // 호스트의 시계 — `drive::run_at`에 넘긴다.
        View::fragment((
            View::component::<ElmView<Editor>>(ElmInput::new(EditorProps::default())),
            View::component::<ElmView<Clock>>(ElmInput::new(ClockProps::default())),
            View::component::<ElmView<Profile>>(ElmInput::new(ProfileProps::default())),
        ))
    }
}

/// 호스트가 프레임마다 부르는 구동 루틴 — 테스트의 `advance`/`press_key`에 해당한다.
#[allow(dead_code)]
fn tick(view: &ElmView<Clock>, started: std::time::Instant) -> bool {
    let mut ctx = view.ctx_mut();
    drive::run_at(&mut ctx, started.elapsed().as_millis() as u64)
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn shortcuts_dispatch_through_the_arena() {
        let mut ctx = Ctx::new();
        let props = EditorProps::default();
        let _ = elm_magic::frame::<Editor>(&mut ctx, &props);

        assert!(drive::dispatch_key(&mut ctx, "Ctrl+S"), "on_key가 등록돼야 한다");
        assert!(!drive::dispatch_key(&mut ctx, "Ctrl+Q"), "없는 키는 false");

        let tree = elm_magic::frame::<Editor>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("saved"));
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

    #[test]
    fn mount_effect_runs_when_driven() {
        let mut ctx = Ctx::new();
        let props = ProfileProps::default();
        let _ = elm_magic::frame::<Profile>(&mut ctx, &props);
        assert!(drive::run(&mut ctx), "on_mount 효과가 실행돼야 한다");
        let tree = elm_magic::frame::<Profile>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("user: user-1"));
    }
}
