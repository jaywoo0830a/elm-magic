//! gpui 예제 05 — 이벤트 → 상태 → 재렌더 (누가 프레임을 다시 도는가)
//!
//! **언제 쓰나**: "클릭했는데 화면이 안 바뀐다"를 이해하거나, 렌더를 줄이고 싶을 때.
//!
//! **동작 순서** (`crates/elm-magic-gpui/src/lib.rs`)
//! 1. `ElmView::render(window, cx)`가 `elm_magic::frame::<C>(ctx, props)`로 트리를 만들고
//!    `Builder`로 gpui 요소를 만든다.
//! 2. 버튼/입력/탭에 붙은 핸들러는 `cx.listener(..)`다 — 즉 **그 엔티티**에 등록된다.
//! 3. 핸들러는 `arena`를 갱신한 뒤 `cx.notify()`를 부른다 → 그 `ElmView`만 다시 렌더된다.
//! 4. 그래서 상태 하나가 바뀌면 **그 화면 전체가** 다시 그려진다(가상 DOM diff는 없다).
//!    노드 수 × 3회 스타일 해석이 프레임 비용이라는 사실이 여기서 나온다
//!    (`crates/elm-magic-egui/benchmark/README.md` — 어댑터 공통).
//!
//! **베스트 패턴 (렌더 줄이기)**
//! - 자주 바뀌는 값(입력 중 텍스트, 애니메이션 값)을 **작은 `ElmView`로 분리**한다.
//!   큰 화면 하나에 넣으면 타이핑마다 전체가 다시 그려진다.
//! - 같은 엔티티를 여러 트리 위치에 붙이지 않는다(엔티티는 하나의 부모 창에 산다).
//! - 무거운 파생 계산은 슬롯이 아니라 **렌더 밖**에서(예: `<-` 효과가 결과를 담게) 한다.
//!
//! **주의**: `ElmView`의 `props`는 생성 시 고정이다(예제 02). props가 자주 바뀌면
//! 엔티티를 매번 새로 만들지 말고 전역 상태로 옮기는 편이 싸다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

/// 자주 바뀌는 부분만 담은 작은 화면 — 여기만 다시 그려진다.
elm_magic::view! {
    fn Draft(value = String::new()) {
        <Col>
            <Input value={value.clone()} on_change={value = _} />
            "draft: {value}"
        </Col>
    }
}

elm_magic::view! {
    fn Counter(n = 0) {
        <Col>
            "Count: {n}"
            <Button on_click={n += 1}>"+"</Button>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(cx.new(ElmView::<Counter>::new))
            // Draft의 타이핑은 Draft 엔티티만 깨운다 → Counter는 다시 그리지 않는다.
            .child(cx.new(ElmView::<Draft>::new))
    }
}

/// 프레임을 강제로 돌려야 할 때(예: 테마 변경)는 최상위 엔티티를 깨운다.
fn refresh_all<C: elm_magic::Component + 'static>(
    view: gpui_kit::Entity<ElmView<C>>,
    cx: &mut gpui_kit::App,
) {
    cx.notify(view.entity_id());
}