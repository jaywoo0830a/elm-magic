//! windows-reactor 예제 05 — 입력: `on_change`는 되고 `on_enter`는 안 된다 (그리고 대안)
//! **업스트림 대응**: `controlled`(controlled 입력) · `form`(`ElementRef` 포커스) ·
//! `calculator`(Enter를 `KeyAccelerators`로 잡기) · `text-box-border`/`text-trimming`.
//!
//!
//! **언제 쓰나**: 텍스트 입력을 가진 화면.
//!
//! **사실 관계** (windows-reactor 0.100.0 `generated.rs`)
//! - `TextBox`가 가진 이벤트는 `on_text_changed`뿐이다. **키 이벤트가 없다** —
//!   그래서 elm의 `on_enter`는 이 어댑터가 붙이지 않는다(계획에는 `enter`로 남는다).
//! - Enter는 **WinUI 수준에서** 잡는다. 업스트림 `calculator` 샘플이 쓰는 바로 그
//!   방법이다 — `Grid::key_accelerators(KeyAccelerators::new([KeyAccelerator::new(
//!   AcceleratorKey::Enter, AcceleratorModifiers::None, context.message(..))]))`.
//!   단 `ElmView`는 `ViewContext`를 노출하지 않으므로 **값을 호스트가 소유**해야
//!   그 콜백에서 제출할 수 있다 — 이 파일의 두 번째 예(`QuickHost`)가 그 모양이다.
//!   `AcceleratorKey`(0.100.0)가 아는 키는 `R` · `Enter` · 사칙연산 · 숫자패드뿐이다:
//!   `Ctrl+S` 같은 문자 키는 **없다**(예제 12의 단축키 주의 참고).
//! - 가장 단순한 대안은 **확인 버튼**이다 — 이 파일의 첫 예(`Form`).
//! - `ElementRef<TextBox>::request_focus()`(업스트림 `form` 샘플)로 포커스를 옮길 수
//!   있지만, 그 `TextBox`는 어댑터가 `<Input>`에서 만드는 것이라 **호스트가 참조를
//!   가질 수 없다**. 포커스 제어가 필요하면 `<Raw>`로 `TextBox`를 직접 만든다(예제 08).
//! - 값은 **elm 슬롯**으로 둔다(`value={name.clone()}`) — WinUI 쪽에 별도 버퍼를
//!   두지 않는다. 그래야 헤드리스 테스트(`type_into`)와 앱이 같은 계약을 쓴다.
//!   (두 번째 예는 반대로 **호스트가 값의 주인**이다 — Enter 때문이다. 둘 중 하나를
//!   고르는 기준은 "키보드 단축키가 필요한가"다.)
//!
//! **`<TextArea>`**: 어댑터가 `accepts_return(true)` + `text_wrapping(Wrap)`을 켠다 —
//! Enter가 개행이 된다(줄바꿈이 곧 입력이므로 `on_enter`가 필요 없다).
//!
//! **주의**: `TextBox`는 Reactor가 **controlled**로 다룬다. elm 상태를 매 프레임
//! `text(..)`로 밀어 넣으므로, 상태를 갱신하지 않으면 사용자가 친 글자가 되돌아간다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{
    AcceleratorKey, AcceleratorModifiers, App, ChildrenControl, Component, ComponentContext, Grid,
    KeyAccelerator, KeyAccelerators, View, ViewContext,
};

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

// ── 두 번째 예: Enter를 WinUI 가속기로 잡는다 ──────────────────────────────
//
// **호스트가 값의 주인**이다. elm은 그 값을 그리고 변경을 콜백 prop으로 올려보낸다.
// 그러면 호스트의 `KeyAccelerator` 콜백이 아레나 없이 제출할 수 있다.

elm_magic::view! {
    fn QuickAdd(value: String = String::new(), submitted: Vec<String> = vec![], on_name: fn(String)) {
        <Col>
            <Input value={value.clone()} on_change={value = _; on_name(value)} />
            <Text>"값: {value}"</Text>
            "제출 {submitted.len()}건"
            <For each={submitted} as={s}>
                <Text>"{s}"</Text>
            </For>
        </Col>
    }
}

#[derive(Clone)]
enum QuickMessage {
    /// elm의 콜백 prop이 큐에 넣는다.
    Name(String),
    /// Enter(또는 호스트 버튼) — 값이 호스트에 있으므로 아레나가 필요 없다.
    Submit,
}

struct QuickHost {
    name: String,
    submitted: Vec<String>,
}

impl Component for QuickHost {
    type Input = ();
    type Message = QuickMessage;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            name: String::new(),
            submitted: Vec::new(),
        }
    }

    fn update(&mut self, message: QuickMessage, _context: &ComponentContext<Self>) {
        match message {
            QuickMessage::Name(value) => self.name = value,
            QuickMessage::Submit => self.submitted.push(self.name.clone()),
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 입력 + Enter");

        // Enter — `TextBox`에 키 이벤트가 없어도 WinUI 가속기는 동작한다
        // (업스트림 `calculator` 샘플이 숫자패드/Enter에 쓰는 방식 그대로).
        let accelerators = KeyAccelerators::new([KeyAccelerator::new(
            AcceleratorKey::Enter,
            AcceleratorModifiers::None,
            context.message(QuickMessage::Submit),
        )]);

        // elm → 호스트: 값 변경을 메시지 큐로 올린다 (예제 11의 계약).
        let sender = context.sender();
        let on_name = Callback::new(move |_arena, value: String| {
            let _ = sender.send(QuickMessage::Name(value));
        });

        Grid::new()
            .key_accelerators(accelerators)
            .children([View::component::<ElmView<QuickAdd>>(ElmInput::new(
                QuickAddProps {
                    value: Some(self.name.clone()),
                    submitted: Some(self.submitted.clone()),
                    on_name: Some(on_name),
                    ..Default::default()
                },
            ))])
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
    // 창 두 개 — 각 창이 자기 ElmView/아레나를 갖는다. 하나만 원하면 `App::run_component`.
    // (공유가 필요하면 예제 13처럼 **루트 컴포넌트 하나**로 묶는다 — 아레나는 창마다 하나다.)
    App::run_windows([
        View::component::<Root>(()),
        View::component::<QuickHost>(()),
    ])
    .expect("Reactor 실행 실패");
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

    /// Enter 경로: **호스트가 값의 주인**이라 아레나 없이 제출한다 — elm은 값과
    /// 변경 콜백 prop만 갖는다. 계획 층에서 확인할 수 있는 것은 "값이 화면에 실리고
    /// 입력이 하나"라는 계약이다.
    #[test]
    fn a_host_owned_input_reports_changes_and_renders_submissions() {
        let mut ctx = Ctx::new();
        let props = QuickAddProps {
            value: Some("elm".to_string()),
            submitted: Some(vec!["elm".to_string()]),
            on_name: Some(Callback::new(|_: &mut Arena, _: String| {})),
            ..Default::default()
        };
        let tree = elm_magic::frame::<QuickAdd>(&mut ctx, &props);
        let (node, pass) = plan(&tree);

        assert_eq!(pass.inputs, 1);
        let input = node
            .children
            .iter()
            .find(|c| matches!(c.kind, PlanKind::TextBox { .. }))
            .expect("Input");
        assert_eq!(input.value.as_deref(), Some("elm"), "값은 호스트가 준다");
        assert!(pass.has_text("값: elm"));
        assert!(pass.has_text("elm"), "제출 목록이 화면에 실린다");
    }
}
