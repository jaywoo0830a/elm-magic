//! gpui 예제 17 — 헤드리스 창 테스트 (`#[gpui_kit::test]` + `TestAppContext`)
//!
//! **언제 쓰나**: 렌더 루프가 **실제 gpui 창에서** 도는지, 두 `ElmView`가 공존해도
//! 안정적인지 같은 어댑터 수준 계약을 CI에서 확인할 때.
//!
//! **준비**
//! - `gpui-kit`을 **dev-dependencies에서** `features = ["test-support"]`로 켠다
//!   (프로덕션 그래프에 들어가지 않게).
//! - `test-support`를 켜면 `use gpui_kit::*;` 글로브가 GPUI의 `test` 속성을 가져와
//!   내장 `#[test]`를 **가린다** — 그래서 리포지토리 테스트도 글로브 대신 쓰는 것만
//!   가져온다.
//!
//! **계약** (`crates/elm-magic-gpui/tests/adapter.rs`가 고정한 것)
//! - `cx.update(gpui_kit::init)`로 테마를 먼저 초기화한다(어댑터 팔레트의 출처).
//! - `window.draw(cx).clear(cx)` + `window.render_frame(cx)`로 프레임을 그린다.
//! - 같은 엔티티로 여러 프레임을 그려도 panic 없이 안정적이어야 한다.
//!
//! **한계 (문서화된 것)**
//! - 어댑터는 요소에 `ElementId`를 등록하지 않으므로 `window.click("id")` 같은
//!   입력 시뮬레이션은 지금 할 수 없다 — 상태 계약은 `mount!`(헤드리스)가 담당한다.
//!   입력까지 어댑터 수준에서 검증하려면 어댑터에 `ElementId`를 추가해야 한다.
//! - 그래서 이 예제는 **렌더 루프 안정성**까지만 확인한다(정직한 범위).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    div, AppContext as _, Context, IntoElement, ParentElement as _, Render, TestAppContext, Window,
};

elm_magic::view! {
    fn GpuiCounter(n = 0) {
        <Col>
            "Count: {n}"
            <Button on_click={n += 1}>"+"</Button>
            <Button on_click={n -= 1}>"-"</Button>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<GpuiCounter>::new))
    }
}

/// 어댑터는 팔레트를 `ActiveTheme`에서 읽으므로 창을 열기 전에 테마를 초기화한다.
fn init_theme(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
}

#[gpui_kit::test]
fn elm_view_renders_in_a_headless_window(cx: &mut TestAppContext) {
    init_theme(cx);
    let handle = cx.add_window(|_, _| Root);
    cx.update_window(handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        window.render_frame(cx);
    })
    .expect("headless window");
}

#[gpui_kit::test]
fn elm_view_survives_repeated_frames(cx: &mut TestAppContext) {
    init_theme(cx);
    let handle = cx.add_window(|_, _| Root);
    cx.update_window(handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        window.render_frame(cx);
        for _ in 0..3 {
            window.render_frame(cx);
        }
    })
    .expect("headless window");
}

#[gpui_kit::test]
fn two_elm_views_coexist(cx: &mut TestAppContext) {
    struct TwoRoot;
    impl Render for TwoRoot {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(cx.new(ElmView::<GpuiCounter>::new))
                .child(cx.new(ElmView::<GpuiCounter>::new))
        }
    }

    init_theme(cx);
    let handle = cx.add_window(|_, _| TwoRoot);
    cx.update_window(handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        window.render_frame(cx);
    })
    .expect("headless window");
}