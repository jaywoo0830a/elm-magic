//! windows-reactor 예제 20 — 실전 골격 (부트스트랩 + 앱 루프 + 전역 상태 + 창 제목)
//!
//! **언제 쓰나**: WinUI 3 앱을 새로 시작할 때 복사하는 뼈대.
//!
//! **순서 (중요)**
//! 1. **스타일 설치가 없다** — 이 어댑터는 스타일 계층을 지원하지 않는다.
//!    테마/색은 WinUI 리소스(`ThemeBrush`)가 담당한다(예제 16).
//! 2. `App::run_component::<Shell>(())` → `Shell::view`가 elm 화면을 붙인다.
//! 3. 창 제목/크기는 **호스트 컴포넌트의 `view()`**에서 선언한다
//!    (`ViewContext::window_title`, `ViewContext::on_window_size`) — elm은 플랫폼을 모른다.
//! 4. **앱이 구동해야 하는 것**: due 효과(`drive::run`), 스트림(`drive::pump_streams`),
//!    시계(`drive::run_at`), 단축키(`drive::dispatch_key`).
//!    `ElmView::update`가 효과/스트림을 자동으로 구동한다(예제 09, 10).
//! 5. 여러 창이 필요하면 `App::run_windows([..])` — 각 항목이 **독립 창**이다.
//!
//! **베스트 패턴**
//! - 셸(창/제목/전역)은 **호스트 컴포넌트** 하나로, 화면 내용은 **elm 루트 컴포넌트**
//!   하나로 나눈다(공유가 필요하면 루트가 하나여야 한다 — 예제 13).
//! - 호스트가 소유한 값(온라인 여부, 로그인 사용자)은 **props로 내려보낸다**(예제 11).
//! - elm 쪽 진실은 `#[store]`로 묶고, 파생 값은 렌더에서 계산한다.
//!
//! **주의**: 효과 본문은 UI 스레드에서 동기 실행된다(`block_on`) — 무거운 일은
//! 호스트의 `spawn_background`로 돌리고 결과만 props로 내려보낸다(예제 11).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{drive, ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

#[store]
struct App {
    dark: bool,
    clicks: i32,
}

elm_magic::view! {
    fn Header() {
        <Row>
            <Strong>"elm-magic"</Strong>
            <Button on_click={app.dark = !app.dark}>"테마"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn Counter() {
        <Col>
            "clicks: {app.clicks}"
            <Button on_click={app.clicks += 1}>"클릭"</Button>
        </Col>
    }
}

elm_magic::view! {
    fn Root(online: bool) {
        on_key("Ctrl+R") { app.clicks = 0 }
        <Col>
            <Header />
            <Counter />
            "online: {online}"
        </Col>
    }
}

/// 호스트 — 창/제목/시계를 소유하고, elm 화면에 값을 props로 내려보낸다.
struct Shell {
    started: std::time::Instant,
    online: bool,
}

impl Component for Shell {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            started: std::time::Instant::now(),
            online: true,
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        // 창 제목은 호스트가 정한다 (elm은 플랫폼을 모른다).
        context.window_title("elm-magic — 앱");

        let _ = self.started; // 프레임마다 `drive::run_at`에 넘길 시계
        View::component::<ElmView<Root>>(ElmInput::new(RootProps {
            online: Some(self.online),
            ..Default::default()
        }))
    }
}

/// 앱 루프의 구동 절반 — 호스트가 프레임마다 부른다.
#[allow(dead_code)]
fn tick(view: &ElmView<Root>, started: std::time::Instant) -> bool {
    let mut ctx = view.ctx_mut();
    let now = started.elapsed().as_millis() as u64;
    let effects = drive::run_at(&mut ctx, now);
    let keys = drive::dispatch_key(&mut ctx, "Ctrl+R");
    effects || keys
}

fn main() {
    // ① 창 하나.
    App::run_component::<Shell>(()).expect("Reactor 실행 실패");

    // ② 창 여러 개 — 각 항목이 독립 창이고, 각자 자기 ElmView/아레나를 갖는다.
    // App::run_windows([
    //     View::component::<ElmView<Root>>(ElmInput::new(RootProps::default())),
    //     View::component::<ElmView<Counter>>(ElmInput::new(CounterProps::default())),
    // ])
    // .expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn shell_renders_the_elm_root_with_host_props() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Root>(
            &mut ctx,
            &RootProps {
                online: Some(false),
                ..Default::default()
            },
        );
        let (_, pass) = plan(&tree);

        assert!(pass.has_text("elm-magic"));
        assert!(pass.has_text("clicks: 0"));
        assert!(pass.has_text("online: false"), "호스트 props가 화면에 반영된다");
    }

    #[test]
    fn store_state_is_shared_inside_the_root() {
        let mut app = elm_magic::mount!(Root);
        app.click("클릭");
        app.assert_text("clicks: 1");
        app.click("테마"); // store 갱신 (화면에는 표시되지 않지만 상태는 바뀐다)
        app.assert_text("clicks: 1");
    }

    #[test]
    fn shortcut_resets_the_counter() {
        let mut ctx = Ctx::new();
        let props = RootProps::default();
        let _ = elm_magic::frame::<Root>(&mut ctx, &props);

        assert!(drive::dispatch_key(&mut ctx, "Ctrl+R"));
        let tree = elm_magic::frame::<Root>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("clicks: 0"));
    }
}
