//! gpui 예제 01 — 최소 카운터 (`ElmView` + `gpui_kit::init`)
//!
//! **언제 쓰나**: gpui-kit 앱에 elm-magic 화면 하나를 붙이는 최소 골격.
//! egui 예제 01과 **같은 컴포넌트**가 그대로 동작한다 — 컴포넌트는 플랫폼을 모른다.
//!
//! **핵심**
//! - `gpui_kit::init(cx)`를 **창을 열기 전에** 부른다. 어댑터의 팔레트는
//!   `ActiveTheme`에서 오므로, 초기화를 건너뛰면 테마 토큰이 기본값으로 떨어진다
//!   (`crates/elm-magic-gpui/tests/adapter.rs`의 `init_theme`).
//! - `ElmView::<C>::new(cx)`가 gpui `Render`를 구현한다 → 트리에 붙이기만 하면 끝.
//! - 상태 아레나는 `ElmView` 엔티티가 소유한다. 그래서 **엔티티 하나 = 컴포넌트 인스턴스
//!   하나**이고, 같은 `ElmView`를 두 번 붙이면 상태도 두 벌이 된다(예제 04).
//!
//! **의존성**: `gpui-kit = "0.6"`만 있으면 된다(GPUI 자체는 재수출된다).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::view! {
    fn Counter(n = 0) {
        <Col>
            "Count: {n}"
            <Row>
                <Button on_click={n -= 1}>"-"</Button>
                <Button on_click={n += 1}>"+"</Button>
            </Row>
        </Col>
    }
}

struct Root;

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // `ElmView::<Counter>::new`는 `fn(&mut Context<ElmView<Counter>>) -> ElmView<Counter>`다.
        div().child(cx.new(ElmView::<Counter>::new))
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx); // 테마/컴포넌트 레이어 초기화 (필수)
        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Root))
                .expect("failed to open window");
        })
        .detach();
    });
}