//! windows-reactor 예제 05 — 입력: `on_change` + `on_enter`(가속기)
//!
//! **업스트림 대응**: `controlled`(controlled 입력) · `calculator`(Enter를
//! `KeyAccelerators`로 잡기) · `form`(`ElementRef` 포커스) · `text-box-border`/`text-trimming`.
//!
//! **언제 쓰나**: 텍스트 입력을 가진 화면.
//!
//! **사실 관계** (windows-reactor 0.100.0 `generated.rs`)
//! - `TextBox`가 가진 이벤트는 `on_text_changed`뿐이다. **키 이벤트가 없다** —
//!   Enter는 WinUI **가속기**(`KeyAccelerators`)로만 잡힌다. 업스트림 `calculator`
//!   샘플이 쓰는 바로 그 방법이다.
//! - 그래서 **어댑터가 대신 붙인다**: elm `<Input on_enter={..}>`의 핸들러를
//!   `AcceleratorKey::Enter` 가속기로 옮기고(`Grid`로 한 겹 감싼다 — 가속기를 받는
//!   컨트롤이 `Grid`/`Button`뿐이다), 눌리면 `ElmMessage::Key`로 돌아와 아레나에서
//!   실행된다. **호스트 코드가 필요 없다**(아래 첫 예 `Form`). `Windows API`의
//!   `KeyAccelerator`를 직접 다룰 일은 이제 없다.
//! - elm `on_key("Ctrl+R")` 같은 선언도 같은 방식으로 매핑된다. 다만
//!   `AcceleratorKey`(0.100.0)가 아는 키는 `R` · `Enter` · 사칙연산 · 숫자패드 +
//!   `Ctrl`뿐이라 **`Ctrl+S`는 매핑되지 않는다** — 그런 키는 호스트가
//!   `KeyAccelerators`를 직접 붙이거나 `<Raw>`에서 처리한다(예제 12의 표 참고).
//! - 값은 **elm 슬롯**으로 둔다(`value={name.clone()}`) — WinUI 쪽에 별도 버퍼를
//!   두지 않는다. 그래야 헤드리스 테스트(`type_into`/`press_enter`)와 앱이 같은
//!   계약을 쓴다. 반대로 **호스트가 값의 주인**이어야 하는 경우(검증/저장을 호스트가
//!   맡는 경우)도 있다 — 두 번째 예(`QuickHost`)가 그 모양이다(거기서는 elm이 값을
//!   갖지 않으므로 어댑터가 붙일 Enter 핸들러도 없다 → 호스트가 직접 단다).
//! - `ElementRef<TextBox>::request_focus()`(업스트림 `form` 샘플)로 포커스를 옮길 수
//!   있지만, 그 `TextBox`는 어댑터가 `<Input>`에서 만드는 것이라 **호스트가 참조를
//!   가질 수 없다**. 포커스 제어가 필요하면 `<Raw>`로 `TextBox`를 직접 만든다(예제 08).
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
            <Input
                value={name.clone()}
                on_change={name = _}
                on_enter={submitted.push(name.clone())} />
            <TextArea value={memo.clone()} on_change={memo = _} />
            <Button on_click={submitted.push(name.clone())}>"제출"</Button>
            <For each={submitted} as={s}>
                <Text>"{s}"</Text>
            </For>
        </Col>
    }
}

// ── 두 번째 예: 값을 **호스트가 소유**하고 가속기도 호스트가 단다 ────────────
//
// elm은 값을 갖지 않고(그래서 `on_enter` 선언도 없다) 변경을 콜백 prop으로 올려보낸다.
// 그러면 호스트의 `KeyAccelerator` 콜백이 아레나 없이 제출할 수 있다 — 검증·저장을
// 호스트가 맡는 앱에서 자연스러운 모양이다(예제 11의 계약).

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

    /// Enter는 **어댑터가** 가속기로 옮긴다 — 계획에 `enter`가 실리고, 헤드리스에서는
    /// `press_enter()`가 그 핸들러를 돈다(앱에서는 WinUI 가속기가 같은 핸들러를 부른다).
    #[test]
    fn enter_is_carried_in_the_plan_and_fires_headless() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Form>(&mut ctx, &FormProps::default());
        let (node, _) = plan(&tree);
        let input = node
            .children
            .iter()
            .find(|c| matches!(c.kind, PlanKind::TextBox { multiline: false }))
            .expect("Input");
        assert!(input.enter.is_some(), "어댑터가 Enter를 가속기로 붙인다");

        let mut app = elm_magic::mount!(Form);
        app.type_into("input", "elm");
        app.press_enter();
        app.assert_text("elm"); // 제출 목록에 들어갔다
    }
}
