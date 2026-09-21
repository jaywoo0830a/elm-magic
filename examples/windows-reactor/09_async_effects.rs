//! windows-reactor 예제 09 — 비동기 효과 `<-` 와 **누가 실행하는가**
//!
//! **언제 쓰나**: 마운트/클릭 뒤에 값을 받아와 상태에 넣을 때.
//!
//! **가장 중요한 사실**
//! - `<-`가 예약한 효과는 **어댑터가 자동으로 실행하지 않는다**. Reactor에서는
//!   `ElmView::update`가 메시지 하나를 처리한 뒤 `drive::run`으로 구동한다
//!   (그래서 클릭 한 번 = 상태 갱신 + 화면 갱신 한 번).
//! - 효과 본문은 `crate::runtime::block_on`으로 **호출 스레드에서 동기 실행**된다
//!   (`src/runtime.rs`). 즉 Reactor의 UI 스레드에서 돈다 — **가벼운 일만** 넣는다.
//!   실제 I/O는 호스트가 맡고 결과만 elm으로 밀어 넣는다(예제 11).
//! - `<- f() after 300ms`는 **시계**가 밀려야 due가 된다:
//!   `drive::run_at(ctx, elapsed_ms)`. Reactor 0.100.0에는 타이머 API가 없으므로
//!   앱이 시계를 공급한다(예: `std::time::Instant` 경과 시간).
//!
//! **규칙**
//! - `slot <- f(args)` — `f`는 `async fn(f_args) -> slot 타입`.
//! - 여러 타깃: `status, users <- load_users()` (튜플 반환 순서대로).
//! - `on_mount { .. }` — 인스턴스가 붙을 때 한 번.
//!
//! **낙관적 업데이트**: `count += 1;`을 **먼저**, `<-`를 나중에 — UI가 즉시 반응하고
//! 응답이 오면 덮어쓴다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{drive, ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

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
                <Text>"{u}"</Text>
            </For>
        </Col>
    }
}

elm_magic::view! {
    fn Likes(count = 0) {
        <Col>
            <Button on_click={
                count += 1;                      // ① 즉시 반영
                count <- optimistic_like(count)  // ② 응답이 오면 덮어씀
            }>"like"</Button>
            "likes: {count}"
        </Col>
    }
}

struct Root;

impl Component for Root {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 효과");
        View::fragment((
            View::component::<ElmView<Users>>(ElmInput::new(UsersProps::default())),
            View::component::<ElmView<Likes>>(ElmInput::new(LikesProps::default())),
        ))
    }
}

/// 지연 효과가 필요하면 앱이 시계를 공급한다 (`drive::run_at`).
#[allow(dead_code)]
fn drive_with_clock(view: &ElmView<Users>, started: std::time::Instant) {
    let elapsed_ms = started.elapsed().as_millis() as u64;
    drive::run_at(&mut view.ctx_mut(), elapsed_ms);
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn mount_effect_fills_state_when_driven() {
        let mut ctx = Ctx::new();
        let props = UsersProps::default();

        let tree = elm_magic::frame::<Users>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("status: idle"), "구동 전");

        assert!(drive::run(&mut ctx), "due 효과가 실행돼야 한다");
        let tree = elm_magic::frame::<Users>(&mut ctx, &props);
        let pass = plan(&tree).1;
        assert!(pass.has_text("status: done"));
        assert!(pass.has_text("alice"));
    }

    #[test]
    fn optimistic_value_wins_before_the_effect_runs() {
        let mut app = elm_magic::mount!(Likes);
        app.click("like");
        app.expect_text("likes: 1"); // flush 전 = 낙관적 값
        app.flush();
        app.expect_text("likes: 10"); // 서버 값
    }
}
