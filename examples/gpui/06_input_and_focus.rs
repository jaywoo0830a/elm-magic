//! gpui 예제 06 — 입력과 포커스 (gpui `FocusHandle` 위임)
//!
//! **언제 쓰나**: 텍스트 입력을 가진 화면. gpui에서 입력은 **포커스 핸들**이 있어야
//! 키 이벤트를 받는다 — 어댑터가 이 부분을 대신 해 준다.
//!
//! **어댑터가 하는 일** (`crates/elm-magic-gpui/src/lib.rs`)
//! - 입력 필드마다 `FocusHandle`을 만들고 `Vec`에 담아 화면 순서를 안정적으로 유지한다
//!   (`ElmView::focus`). 그래서 리렌더 사이에도 같은 입력이 같은 핸들을 유지한다.
//! - 그려진 입력 수는 `Pass::inputs`에 기록된다 → 검증/진단용.
//! - `<Input on_change={..} on_enter={..} />`의 `on_change`는 `cx.listener`로 들어가
//!   키 입력마다 아레나를 갱신하고 `cx.notify()`로 프레임을 다시 돈다.
//!
//! **베스트 패턴**
//! - 입력값은 **상태 슬롯**으로 둔다(`value={query.clone()}`) — gpui 쪽에 별도 버퍼를
//!   두지 않는다. 그래야 헤드리스 테스트(`type_`)와 앱이 같은 계약을 쓴다.
//! - `<Raw>`로 gpui `Window`를 만질 수 있긴 하지만, 포커스는 어댑터에 맡기는 편이
//!   안전하다(직접 만들면 리렌더마다 핸들이 새로 생겨 포커스가 튄다).
//! - 붙자마자 포커스를 주고 싶으면 `<Raw>|window: &mut gpui_kit::Window| { … }</Raw>`보다
//!   사용자가 클릭하게 두는 편이 예측 가능하다.
//!
//! **주의**: `text-transform`은 `Input` 라벨이 아니라 표시 텍스트에 적용된다 —
//! 입력값 자체를 변형하려면 상태에서 변형한 값을 `value`로 넘긴다(예제 15 참고).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::view! {
    fn Form(
        name = String::new(),
        memo = String::new(),
        submitted: Vec<String> = vec![],
    ) {
        <Col>
            <Input
                class="field"
                value={name.clone()}
                on_change={name = _}
                on_enter={submitted.push(name.clone())} />
            <TextArea
                class="field"
                value={memo.clone()}
                on_change={memo = _} />
            <Button on_click={submitted.push(name.clone())}>"제출"</Button>
            <For each={submitted} as={s}>
                <Row>"{s}"</Row>
            </For>
        </Col>
    }
}

elm_magic::css! {
    .field { padding: 6 8; radius: 6; border-width: 1; border-color: border; bg: background; }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<Form>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_gpui::ElmView as View;

    /// 어댑터를 띄우지 않고 계약부터 고정한다(가장 빠른 피드백).
    #[test]
    fn form_contract_is_platform_independent() {
        let mut app = elm_magic::mount!(Form);
        app.type_into("input", "elm");
        app.press_enter();
        app.assert_text("elm");
    }

    /// 어댑터가 입력 수를 세는지(포커스 핸들을 만들었다는 신호) 확인하는 자리.
    /// 실제 창에서 확인하려면 `#[gpui_kit::test]` + `TestAppContext`(예제 17).
    #[allow(dead_code)]
    fn rendered_input_count(view: &View<Form>) -> usize {
        view.pass().inputs
    }
}