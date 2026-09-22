//! windows-reactor 예제 14 — 피드백 위젯이 **WinUI 컨트롤**로 그려진다
//!
//! **업스트림 대응**: `form` · `radio-buttons` · `message-box` — 업스트림이
//! `ProgressBar`/`CheckBox`/`InfoBar`를 쓰는 자리와 **같은 컨트롤을 같은 방식**
//! (controlled)으로 쓴다.
//!
//! **언제 쓰나**: 로딩/진행/체크/배너를 테마에 맞게 보여줄 때. 어댑터가 WinUI 컨트롤을
//! 골라 쓰므로 **추가 코드가 없다**.
//!
//! **매핑** (`crates/elm-magic-windows-reactor/src/plan.rs`)
//! | elm 태그 | WinUI 컨트롤 | 비고 |
//! |---|---|---|
//! | `<Check checked on_change>` | `CheckBox` | `is_checked` + `on_is_checked_changed` (controlled) |
//! | `<Progress value={0..1}>` | `ProgressBar` | 어댑터가 0..100으로 변환 |
//! | `<Spinner />` | `ProgressRing` | `is_active(true)` (애니메이션) |
//! | `<Divider />` | `Border` | 1px 위 테두리 |
//! | `<Banner kind="..">` | `InfoBar` | severity: error/warn/success/info |
//! | `<Text>`/`<Strong>` | `TextBlock` | Strong은 `FontWeight::BOLD` |
//!
//! **핵심**
//! - **controlled 위젯**: 체크/텍스트는 매 프레임 elm 상태를 밀어 넣는다. 상태를
//!   갱신하지 않으면 사용자의 조작이 되돌아간다(예제 05의 주의와 같은 이유).
//! - **`<Banner>`는 상태로 켜고 끈다** — `InfoBar`의 닫기(X)는 네이티브 동작이라
//!   elm 상태를 모른다. 닫힌 뒤에도 그리면 다시 열리므로, 닫기를 다루려면
//!   `<Raw>`로 `InfoBar::on_closed`를 붙여 상태를 갱신해야 한다.
//! - **색/간격은 여기 없다**: `css!`는 이 백엔드에서 효과가 없다(예제 16).
//!   `InfoBar`의 severity, `ProgressBar`의 테마 악센트가 곧 스타일이다.

use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    fn Panel(agreed = false, synced = false, pct = 0.25, tab = 0) {
        <Col>
            <Spinner />
            <Progress value={pct} />
            <Divider />
            <Banner kind="error">"저장하지 못했습니다"</Banner>
            <Banner kind="success">"저장했습니다"</Banner>
            <Banner kind="warn">"곧 만료됩니다"</Banner>
            <Banner kind="info">"동기화 대기 중"</Banner>
            <Check checked={agreed} on_change={agreed = _}>"동의"</Check>
            <Check checked={synced} on_change={synced = !synced}>"동기화"</Check>
            <Row>
                <Tab active={tab == 0} on_click={tab = 0}>"Home"</Tab>
                <Tab active={tab == 1} on_click={tab = 1}>"Stats"</Tab>
            </Row>
            <Row>
                <Th on_click={tab = 1}>"Name"</Th>
                <Td>"cell"</Td>
            </Row>
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
        context.window_title("elm-magic — 피드백 위젯");
        View::component::<ElmView<Panel>>(ElmInput::new(PanelProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::{plan, PlanKind, Severity};

    #[test]
    fn every_feedback_widget_maps_to_a_winui_control() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Panel>(&mut ctx, &PanelProps::default());
        let (node, pass) = plan(&tree);

        assert_eq!(pass.count("ProgressRing"), 1);
        assert_eq!(pass.count("ProgressBar"), 1);
        assert_eq!(pass.count("Border"), 1, "Divider");
        assert_eq!(pass.count("InfoBar"), 4);
        assert_eq!(pass.count("CheckBox"), 2);
        assert_eq!(pass.count("Button"), 3, "Tab 둘 + 정렬 가능한 Th");

        // severity가 종류별로 다르게 실린다.
        let severities: Vec<Severity> = node
            .children
            .iter()
            .filter_map(|c| match c.kind {
                PlanKind::Banner { severity } => Some(severity),
                _ => None,
            })
            .collect();
        assert_eq!(
            severities,
            vec![
                Severity::Error,
                Severity::Success,
                Severity::Warning,
                Severity::Info
            ]
        );
    }

    #[test]
    fn checkbox_state_round_trips() {
        let mut app = elm_magic::mount!(Panel);
        app.toggle("동의");
        assert!(app.render_tree().contains("Check \"동의\" checked=true"));
        app.set_check("동기화", true);
        assert!(app.render_tree().contains("Check \"동기화\" checked=true"));
    }
}
