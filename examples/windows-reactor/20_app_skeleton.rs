//! windows-reactor 예제 20 — 실전 골격 (부트스트랩 + 앱 루프 + 전역 상태 + 창 제목)
//! **업스트림 대응**: `window` · `secondary-window` · `self_contained` ·
//! `lightweight-resources` — 업스트림 샘플들이 나눠서 보여주는 것(창 선언 · 다중 창 ·
//! 배포 형태)을 이 한 파일의 순서로 묶었다.
//!
//!
//! **언제 쓰나**: WinUI 3 앱을 새로 시작할 때 복사하는 뼈대.
//!
//! **순서 (중요)**
//! 1. **스타일 설치가 없다** — 이 어댑터는 스타일 계층을 지원하지 않는다.
//!    테마/색은 WinUI 리소스(`ThemeBrush`)가 담당한다(예제 16).
//! 2. **창은 호스트가 선언한다**: `window_title`, `window_visuals(client_size)`,
//!    `on_window_size` — Reactor가 창을 다루는 자리다(업스트림 `window` 샘플과 같은 API).
//!    elm만으로 끝내려면 같은 선언을 `ElmInput`에 붙인다(예제 01) — **창마다 하나만**
//!    선언한다(호스트와 elm이 같은 창에 둘 다 선언하면 중복이다).
//! 3. elm 화면은 `View::component::<ElmView<Root>>(ElmInput::new(props))`로 붙인다.
//!    (`#[store] struct App`이 이 모듈에 `struct App`을 만들므로 Reactor의 `App`은
//!    import 하지 않고 경로로 쓴다 — `use windows_reactor::App`은 E0255다.)
//! 4. **앱이 구동할 수 있는 것과 없는 것** (정본은 예제 12의 표):
//!    - **돈다**: `<- f()`(지연 없음), `on_mount` / `on_unmount` — `ElmView::update`가
//!      메시지 처리 뒤 같은 발행에서 `drive::run`을 부른다.
//!    - **돈다 (지원 키 한정)**: `on_key` — 어댑터가 마지막 프레임의 `Ctx::keys`를
//!      WinUI `KeyAccelerators`로 매핑해 준다(아래 `Root`의 `Ctrl+R`). `AcceleratorKey`
//!      (0.100.0)에 없는 키는 매핑되지 않으니 호스트가 직접 붙인다.
//!    - **돌지 않는다**: `<- f() after …` · `on_tick` — 호스트가 아레나에 접근할 수
//!      없다(`ElmView` 인스턴스 참조를 얻을 길이 없다, 예제 12). **시간이 흐르는 일은
//!      호스트가 소유**하고(`schedule_tick`) 결과를 props로 내려보낸다.
//! 5. 창을 더 열려면 `ComponentContext::open_window(view)`(런타임, 업스트림
//!    `secondary-window`) 또는 `App::run_windows([..])`(시작할 때) — 창마다 아레나가 따로다.
//!    elm은 창을 만들 수 없으므로 **콜백 prop**으로 호스트에게 부탁한다(아래 `on_open`).
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
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{Component, ComponentContext, View, ViewContext, WindowSize, WindowVisuals};

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
    fn Root(online: bool, on_open: fn()) {
        // 지원 키는 **어댑터가 매핑**해서 앱에서도 돈다(`Ctrl+R` → `AcceleratorKey::R`
        // + `Ctrl`). 매핑 불가 키(`Ctrl+S`)는 호스트 몫이다(예제 05의 두 번째 예).
        on_key("Ctrl+R") { app.clicks = 0 }
        <Col>
            <Header />
            <Counter />
            "online: {online}"
            // 창은 호스트만 열 수 있다 — elm은 콜백 prop으로 부탁한다(예제 11의 계약).
            <Button on_click={on_open()}>"새 창"</Button>
        </Col>
    }
}

/// 호스트 — **창/제목/크기를 소유**하고, elm 화면에 값을 props로 내려보낸다.
/// elm이 창을 열 수 없으므로(`open_window`는 `ComponentContext`에만 있다) elm의
/// "새 창" 요청은 **콜백 prop**으로 올라온다(예제 11의 계약).
struct Shell {
    online: bool,
    windows: u32,
    size: WindowSize,
}

#[derive(Clone)]
enum ShellMessage {
    /// elm의 콜백 prop(`on_open`)이 큐에 넣는다 — 새 창을 연다.
    Open,
    /// 창 크기 변화 — `ViewContext::on_window_size`가 준다.
    Resized(WindowSize),
}

impl Component for Shell {
    type Input = ();
    type Message = ShellMessage;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            online: true,
            windows: 0,
            size: WindowSize {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    fn update(&mut self, message: ShellMessage, context: &ComponentContext<Self>) {
        match message {
            // 여러 창은 `context.open_window`로 **런타임에** 연다
            // (업스트림 `secondary-window` 샘플). 시작할 때 여는 것은 아래 `main`의
            // `App::run_windows([..])`다 — 둘 다 각 창이 독립 `ElmView`/아레나를 갖는다.
            ShellMessage::Open => {
                let opened = context.open_window(View::component::<ElmView<Root>>(ElmInput::new(
                    RootProps {
                        online: Some(self.online),
                        ..Default::default()
                    },
                )));
                if opened {
                    self.windows += 1;
                }
            }
            ShellMessage::Resized(size) => self.size = size,
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        // 창 제목·크기는 호스트가 정한다 (elm은 플랫폼을 모른다).
        context.window_title("elm-magic — 앱");
        context.window_visuals(WindowVisuals::new().client_size(720.0, 480.0));
        // 크기 변화는 콜백으로 호스트 상태에 들어온다 (업스트림 `window` 샘플의 자리).
        context.on_window_size(context.callback(ShellMessage::Resized));

        // elm → 호스트: 창을 여는 일은 호스트만 할 수 있다. 콜백 prop을 내려보내면
        // elm의 버튼이 `on_open()`을 부르고, 호스트는 자기 메시지 큐에 넣는다
        // (Reactor는 콜백 안에서 `update`를 인라인 실행하지 않는다).
        let sender = context.sender();
        let on_open = Callback::new(move |_arena, ()| {
            let _ = sender.send(ShellMessage::Open);
        });

        View::fragment((
            View::component::<ElmView<Root>>(ElmInput::new(RootProps {
                online: Some(self.online),
                on_open: Some(on_open),
                ..Default::default()
            })),
            // 호스트가 소유한 값 — elm 문법으로 표현할 수 없는 것들은 이렇게 내려보낸다.
            format!("창 {}개 · {}×{}", self.windows, self.size.width, self.size.height),
        ))
    }
}

fn main() {
    // ① 창 하나.
    windows_reactor::App::run_component::<Shell>(()).expect("Reactor 실행 실패");

    // ② 창 여러 개 — 각 항목이 독립 창이고, 각자 자기 ElmView/아레나를 갖는다.
    // windows_reactor::App::run_windows([
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
        // `Root(online: bool)`는 필수 prop이라 `mount!`로는 마운트할 수 없다.
        let mut app = elm_magic::mount_with::<Root>(RootProps {
            online: Some(true),
            ..Default::default()
        });
        app.click("클릭");
        app.assert_text("clicks: 1");
        app.click("테마"); // store 갱신 (화면에는 표시되지 않지만 상태는 바뀐다)
        app.assert_text("clicks: 1");
    }

    /// elm의 콜백 prop은 **값 없이** 호스트에게 올라간다(`on_open()`) — 창 열기는
    /// 호스트만 할 수 있으므로(`open_window`는 `ComponentContext`에만 있다) 이 경로가
    /// 유일하다. `zero-arg` 콜백 prop이 `Callback<()>`로 풀리는 것도 함께 고정한다.
    #[test]
    fn the_open_request_reaches_the_host() {
        let opened = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = std::rc::Rc::clone(&opened);
        let mut app = elm_magic::mount_with::<Root>(RootProps {
            online: Some(true),
            on_open: Some(Callback::new(move |_arena, ()| {
                counter.set(counter.get() + 1);
            })),
            ..Default::default()
        });

        app.click("새 창");
        assert_eq!(opened.get(), 1, "elm의 버튼이 호스트 콜백을 부른다");
    }

    #[test]
    fn shortcut_resets_the_counter() {
        let mut app = elm_magic::mount_with::<Root>(RootProps {
            online: Some(true),
            ..Default::default()
        });
        app.click("클릭");
        app.click("클릭");
        app.assert_text("clicks: 2");

        // 호스트가 키 이벤트마다 부르는 것과 같은 경로(`drive::dispatch_key`)로
        // `on_key("Ctrl+R")` 핸들러가 돈다.
        app.press_key("Ctrl+R");
        app.assert_text("clicks: 0");
    }
}
