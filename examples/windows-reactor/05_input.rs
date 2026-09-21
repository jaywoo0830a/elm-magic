//! windows-reactor 예제 05 — 입력: `on_change`는 되고 `on_enter`는 안 된다 (그리고 대안)
//!
//! **언제 쓰나**: 텍스트 입력을 가진 화면.
//!
//! **사실 관계** (Reactor 0.100.0 `generated.rs`)
//! - `TextBox`가 가진 이벤트는 `on_text_changed`뿐이다. **키 이벤트가 없다** —
//!   그래서 elm의 `on_enter`는 이 어댑터가 붙이지 않는다(계획에는 `enter`로 남는다).
//! - Enter가 필요하면 세 가지 중 하나를 쓴다:
//!   1. **확인 버튼**을 둔다(가장 단순, 권장).
//!   2. `<Raw>`로 `Border` + 라우티드 키 콜백을 감싼다(공식 문서의
//!      `on_preview_key_down` + `RoutedMessage::handled` 패턴).
//!   3. 포커스가 빠져나갈 때(`LostFocus`) 값을 확정한다.
//! - 값은 **elm 슬롯**으로 둔다(`value={name.clone()}`) — WinUI 쪽에 별도 버퍼를
//!   두지 않는다. 그래야 헤드리스 테스트(`type_into`)와 앱이 같은 계약을 쓴다.
//!
//! **`<TextArea>`**: 어댑터가 `accepts_return(true)` + `text_wrapping(Wrap)`을 켠다 —
//! Enter가 개행이 된다(줄바꿈이 곧 입력이므로 `on_enter`가 필요 없다).
//!
//! **주의**: `TextBox`는 Reactor가 **controlled**로 다룬다. elm 상태를 매 프레임
//! `text(..)`로 밀어 넣으므로, 상태를 갱신하지 않으면 사용자가 친 글자가 되돌아간다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    fn Form(
        name = String::new(),
        memo = String::new(),
        submitted: Vec<String> = vec![],
    ) {
        <Col>
            <Input value={name.clone()} on_change={name = _} />
            <TextArea value={memo.clone()} on_change={memo = _} />
            <Button on_click={submitted.push(name.clone())}>"제출"</Button>
            <For each={submitted} as={s}>
                <Text>"{s}"</Text>
            </For>
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
        context.window_title("elm-magic — 입력");
        View::component::<ElmView<Form>>(ElmInput::new(FormProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::{plan, PlanEvent, PlanKind};

    #[test]
    fn input_is_a_single_line_text_box_with_a_change_handler() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Form>(&mut ctx, &FormProps::default());
        let (node, pass) = plan(&tree);

        assert_eq!(pass.inputs, 2);
        let input = &node.children[0];
        assert!(matches!(input.kind, PlanKind::TextBox { multiline: false }));
        assert!(matches!(input.event, Some(PlanEvent::Change(_))));
        assert_eq!(input.value.as_deref(), Some(""));

        // `<TextArea>`는 여러 줄 — Enter가 개행이 된다.
        let area = &node.children[1];
        assert!(matches!(area.kind, PlanKind::TextBox { multiline: true }));
    }

    /// 값 계약은 헤드리스로 검증한다 — 어댑터 없이 같은 컴포넌트를 돌린다.
    #[test]
    fn typing_updates_state_and_submit_reads_it() {
        let mut app = elm_magic::mount!(Form);
        app.type_into("input", "elm");
        app.click("제출");
        app.assert_text("elm");
    }
}
