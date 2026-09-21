//! gpui 예제 09 — `<Raw>` 탈출구 (gpui `Window`를 직접 받기)
//!
//! **언제 쓰나**: gpui/gpui-kit에만 있는 것을 한 자리에 넣을 때 — 키 바인딩 등록,
//! 창 제목 변경, gpui-kit 컴포넌트 임베드, 커스텀 페인트.
//!
//! **계약** (사양서 7.3)
//! - 이 어댑터에서 `<Raw>` 클로저는 **`&mut gpui_kit::Window`**를 받는다.
//!   요소를 **돌려줄 수 없다** — 클로저의 반환값은 버려지고, 그 자리에는 빈 `div()`가
//!   들어간다. 즉 gpui에서 `<Raw>`는 **부수 효과 전용**이다.
//! - 헤드리스 테스트에서는 무시되고 `render_tree()`에 `[raw]`로만 보인다.
//! - 페이로드 타입이 다르면 런타임 panic("platform payload type mismatch")이 난다 —
//!   같은 트리를 egui와 gpui에 동시에 쓸 수 없다.
//!
//! **베스트 패턴**
//! - **키 바인딩/전역 단축키**는 Raw가 아니라 gpui `App` 부트스트랩에서 등록한다
//!   (`cx.bind_keys(...)` / `on_action`) — Raw는 프레임마다 불리므로 매번 등록하면
//!   중복된다.
//! - Raw에서 **elm 슬롯을 읽지 않는다**. 필요한 값은 클로저가 캡처(`move`)한다.
//! - Raw가 그리는 것에 대한 계약은 헤드리스에서 못 잡는다. 가능하면 그 조각을
//!   작은 gpui 엔티티로 만들고 `#[gpui_kit::test]`로 확인한다(예제 17).
//! - gpui-kit 컴포넌트를 크게 쓰게 되면, 어댑터에 정식 태그를 추가하는 편이 낫다
//!   (`Widget` 트레이트 — `Bridge` 아이디어는 `0.8-preview.md` §6.4).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::view! {
    fn TitleChanger(title = String::from("elm-magic")) {
        <Col>
            <Raw>|window: &mut gpui_kit::Window| {
                // 창 제목은 gpui Window의 것이고 elm-magic에는 대응 태그가 없다.
                window.set_window_title("elm-magic — Raw 예제");
            }</Raw>
            "{title}"
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<TitleChanger>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Raw의 예상 페이로드 타입이 **플랫폼별로 다르다**는 사실을 문서화하는 테스트.
    /// (여기서는 페이로드 생성이 불가능하므로 헤드리스 계약만 확인한다.)
    #[test]
    fn raw_is_adapter_specific() {
        let app = elm_magic::mount!(TitleChanger);
        app.expect_text("elm-magic");
        assert!(app.render_tree().contains("[raw]"));
    }
}