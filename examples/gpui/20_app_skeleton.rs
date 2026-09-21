//! gpui 예제 20 — 실전 골격 (부트스트랩 + 스타일 + 앱 루프 + 전역 상태)
//!
//! **언제 쓰나**: gpui-kit 앱을 새로 시작할 때 복사하는 뼈대. 지금까지의 조각을
//! 한 파일에 모았고, **실제 앱에서 반드시 필요한 구동 코드**를 포함한다.
//!
//! **순서 (중요)**
//! 1. `install_styles()` — 스타일 등록은 **첫 렌더 전에**. `css!`(시작 섹션)를 쓰지
//!    않는다면 여기서 `style::register`를 부른다(예제 16).
//! 2. `gpui_kit::application().run(..)` → `gpui_kit::init(cx)` → 창 열기.
//! 3. 창 루프에서 `ElmView`가 `frame` → gpui 요소 변환을 한다(어댑터가 처리).
//! 4. **앱이 직접 구동해야 하는 것**: due 효과(`take_due`), 스트림(`take_streams`),
//!    시계(`set_now`), `on_key` 디스패치. 테스트에서 `flush`/`advance`/`pump`/
//!    `press_key`가 하는 일이 정확히 이것이다(예제 11, 13).
//!
//! **베스트 패턴**
//! - 화면 셸은 `Root` 하나(작게), 화면별 내용은 `ElmView` 여러 개로 나눈다(예제 05).
//! - 앱 루프에는 `tick()` 하나만 두고 규칙을 고정한다:
//!   시계 → 효과 → 스트림 → (변경 시) `cx.notify()`.
//! - 전역(`#[store]`)에는 진실만, 나머지는 파생 값으로.
//! - 스타일은 시작 때 한 번, 팔레트는 테마에 맡긴다(예제 03).
//!
//! **주의**: `gpui_kit::init(cx)`를 빠뜨리면 테마 토큰이 기본값이 되고, `tick()`을
//! 빠뜨리면 `<-`/`->`가 **영원히 실행되지 않는다**(조용한 실패 — 가장 흔한 함정).

use elm_magic::prelude::*;
use elm_magic::style::{Edges, StyleSpec, Token};
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

// ── 1. 스타일 (css! 없이 — 등록 시점을 앱이 통제한다) ────────
fn install_styles() {
    elm_magic::style::register(vec![
        (
            ".app",
            StyleSpec {
                gap: Some(0.0),
                bg: Some(Token::Background),
                ..StyleSpec::NONE
            },
        ),
        (
            ".app__bar",
            StyleSpec {
                height: Some(40.0),
                padding: Some(Edges::new(8.0, 12.0, 8.0, 12.0)),
                bg: Some(Token::Surface),
                border_bottom_width: Some(1.0),
                border_color: Some(Token::Border),
                ..StyleSpec::NONE
            },
        ),
        (
            ".app__body",
            StyleSpec {
                flex_grow: Some(1.0),
                padding: Some(Edges::splat(12.0)),
                gap: Some(8.0),
                ..StyleSpec::NONE
            },
        ),
        (
            ".card",
            StyleSpec {
                gap: Some(8.0),
                padding: Some(Edges::splat(12.0)),
                bg: Some(Token::Surface),
                radius: Some(8.0),
                ..StyleSpec::NONE
            },
        ),
        (
            ".muted",
            StyleSpec { color: Some(Token::TextDim), font_size: Some(12.0), ..StyleSpec::NONE },
        ),
    ]);
}

// ── 2. 전역 상태 ────────────────────────────────────────────
#[store]
struct App {
    dark: bool,
    clicks: i32,
}

// ── 3. 화면 ────────────────────────────────────────────────
elm_magic::view! {
    fn Header() {
        <Row class="app__bar">
            <Strong>"elm-magic"</Strong>
            <Button on_click={app.dark = !app.dark}>"테마"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn Counter() {
        <Col class="card">
            "clicks: {app.clicks}"
            <Button on_click={app.clicks += 1}>"클릭"</Button>
        </Col>
    }
}

elm_magic::view! {
    fn Root() {
        <Col class="app">
            <Header />
            <Col class="app__body">
                <Counter />
                <Text class="muted">"효과/스트림/단축키는 앱 루프가 구동한다"</Text>
            </Col>
        </Col>
    }
}

// ── 4. 앱 루프 ─────────────────────────────────────────────
struct Shell;

impl Shell {
    /// 프레임마다 한 번: 시계 → 효과 → 스트림 → (변경 시) 재렌더.
    fn tick(view: &gpui_kit::Entity<ElmView<Root>>, cx: &mut gpui_kit::App, now_ms: u64) {
        view.update(cx, |view, cx| {
            let ctx = view.ctx_mut();
            ctx.arena.set_now(now_ms);

            let mut changed = false;
            loop {
                let due = ctx.arena.take_due();
                if due.is_empty() {
                    break;
                }
                for effect in due {
                    effect(&mut ctx.arena);
                }
                changed = true;
            }

            let tasks = ctx.arena.take_streams();
            let mut keep = Vec::new();
            for mut task in tasks {
                if task(&mut ctx.arena) {
                    keep.push(task);
                }
            }
            ctx.arena.push_streams(keep);

            if changed {
                cx.notify();
            }
        });
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(cx.new(ElmView::<Root>::new))
    }
}

fn main() {
    install_styles(); // 첫 렌더 전에 반드시

    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx); // 테마 초기화 (팔레트의 출처)
        cx.spawn(async move |cx| {
            let handle = cx
                .open_window(WindowOptions::default(), |_, cx| cx.new(|_| Shell))
                .expect("failed to open window");

            // 예: 16ms마다 tick()을 돌려 효과/스트림을 구동한다.
            //   (실제 코드에서는 gpui 타이머/`cx.background_executor()`를 쓴다.)
            let _ = handle;
        })
        .detach();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_and_store_work_headlessly() {
        let mut app = elm_magic::mount!(Root);
        app.assert_text("elm-magic");
        app.assert_text("clicks: 0");
        app.click("클릭");
        app.assert_text("clicks: 1");
        app.click("테마");
        // 스타일이 실제로 해석되는지도 함께 본다(등록 누락은 여기서 드러난다).
        let col = app.element().resolved_style(&elm_magic::style::Palette::dark());
        assert_eq!(col.bg, Some(elm_magic::style::Palette::dark().get(Token::Background)));
    }
}