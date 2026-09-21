//! gpui 예제 04 — 한 창에 여러 `ElmView` (컴포넌트 인스턴스와 상태 격리)
//!
//! **언제 쓰나**: 같은 컴포넌트를 두 곳에 놓거나(대시보드), 서로 다른 컴포넌트를
//! 나란히 놓을 때. 상태가 어떻게 갈리는지 이해하는 것이 핵심이다.
//!
//! **규칙**
//! - **엔티티 하나 = 인스턴스 하나.** `cx.new(ElmView::<C>::new)`를 두 번 부르면
//!   상태 아레나(`Ctx`)도 두 벌이다 → 카운터를 두 개 놓으면 각자 센다.
//! - 자식 `ElmView`들은 부모 gpui 트리 안에서 **독립 엔티티**로 산다. 그래서
//!   한쪽의 `cx.notify()`가 다른 쪽을 다시 그리지 않는다(과잉 렌더가 없다).
//! - 반대로 공유가 필요하면 `#[store]` 전역 상태(예제 14)를 쓴다. 전역 쓰기는
//!   자기 인스턴스만 깨우므로, 두 화면이 같은 전역을 보려면 각각 다시 렌더돼야
//!   한다(부모가 `cx.notify()`를 하거나 프레임이 돌 때 반영).
//!
//! **베스트 패턴**: 화면 단위로 `ElmView`를 쪼개고, 공유는 전역/부모 상태로만 한다.
//! 한 `ElmView`에 모든 것을 몰아넣으면 그 엔티티 전체가 매번 다시 그려진다.
//!
//! **주의**: 리포지토리 테스트가 고정한 사실 — 두 `ElmView`가 같은 창에서 여러
//! 프레임을 그려도 panic 없이 안정적이다(`crates/elm-magic-gpui/tests/adapter.rs`).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::view! {
    fn Counter(n = 0) {
        <Col class="counter">
            "Count: {n}"
            <Button on_click={n += 1}>"+"</Button>
        </Col>
    }
}

elm_magic::view! {
    fn Clock(ticks = 0) {
        on_tick(1s) { ticks += 1 }
        <Col class="clock">
            "ticks: {ticks}"
        </Col>
    }
}

struct Dashboard;
impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    // 상태가 서로 다른 인스턴스 두 개
                    .child(cx.new(ElmView::<Counter>::new))
                    .child(cx.new(ElmView::<Counter>::new)),
            )
            .child(cx.new(ElmView::<Clock>::new))
    }
}