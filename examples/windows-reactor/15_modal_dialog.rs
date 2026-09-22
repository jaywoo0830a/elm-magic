//! windows-reactor 예제 15 — `<Modal>` → `ContentDialog` (**`on_close`가 필수**)
//!
//! **언제 쓰나**: 확인/입력 다이얼로그. Reactor는 WinUI `ContentDialog`를 만들어 준다.
//!
//! **왜 `on_close`가 필수인가**
//! - 어댑터는 매 프레임 `ContentDialog::is_open(true)`를 선언한다(elm 상태가
//!   "열림"이니까).
//! - 사용자가 X/Esc/버튼으로 닫으면 WinUI는 `on_closed(ContentDialogResult)`를 준다.
//!   elm이 그걸 받아 상태를 `false`로 바꾸지 않으면, **다음 프레임에 다시 열린다**.
//! - 그래서 `<Modal on_close={open = false}>`처럼 **닫힘을 상태에 반영**해야 한다.
//!
//! **버튼 문구**: `<Modal>`은 `ContentDialog`의 `title` + 본문 + `on_closed`만 옮긴다.
//! 0.100.0의 `ContentDialog`가 가진 나머지는 `primary_button_text` ·
//! `secondary_button_text` · `close_button_text` · `is_primary_button_enabled` ·
//! `is_secondary_button_enabled` · `on_closed(ContentDialogResult)`다 — 이걸 쓰려면
//! **다이얼로그를 직접** 만들어야 한다(`<Raw>` 또는 호스트 컴포넌트). 그러면 열림
//! 상태도 호스트가 소유한다(버튼 문구를 elm 상태로 바꾸는 길은 없다).
//! 여기서는 다이얼로그 **본문 안에 elm `<Button>`**을 두는 쪽을 쓴다 — 문구가 elm
//! 상태/로케일을 따르고, 닫힘도 elm 상태 하나로 정리된다.
//!
//! **업스트림 대응**: `message-box` 샘플은 `context.run_window`로 네이티브 대화상자를
//! 띄우는데, 그 API는 **배포된 0.100.0에 없다**(업스트림 샘플은 저장소 경로 의존이라
//! 마스터를 따라간다 — `windows-reactor = { workspace = true }`). 이 저장소는 배포본
//! 0.100.0을 쓰므로 WinUI `ContentDialog`를 쓰는 `<Modal>`이 정답이다.
//!
//! **주의**: `ContentDialog`는 한 번에 하나만 열 수 있다(WinUI 규칙). 여러 개를
//! 동시에 열어야 하면 화면 전환(라우팅)이나 `Flyout`으로 푼다.

use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    fn Panel(open = false, accepted = 0, rejected = 0) {
        <Col>
            "accepted: {accepted} / rejected: {rejected}"
            <Button on_click={open = true}>"열기"</Button>
            <Modal title="계속할까요?" on_close={open = false}>
                <Col>
                    <Text>"이 작업은 되돌릴 수 없습니다."</Text>
                    <Row>
                        <Button on_click={open = false; accepted += 1}>"확인"</Button>
                        <Button on_click={open = false; rejected += 1}>"취소"</Button>
                    </Row>
                </Col>
            </Modal>
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
        context.window_title("elm-magic — 다이얼로그");
        View::component::<ElmView<Panel>>(ElmInput::new(PanelProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic::prelude::*;
    use elm_magic_windows_reactor::{plan, PlanEvent};

    #[test]
    fn modal_is_a_content_dialog_with_a_close_event() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Panel>(&mut ctx, &PanelProps::default());
        let (node, pass) = plan(&tree);

        assert_eq!(pass.count("ContentDialog"), 1);
        assert_eq!(pass.dialogs, 1);

        let dialog = node
            .children
            .iter()
            .find(|c| c.control() == "ContentDialog")
            .expect("ContentDialog");
        assert_eq!(dialog.text, "계속할까요?");
        assert!(
            matches!(dialog.event, Some(PlanEvent::Close(_))),
            "on_close가 있어야 닫힘이 상태에 반영된다"
        );
    }

    #[test]
    fn closing_the_dialog_updates_state() {
        let mut app = elm_magic::mount!(Panel);
        app.click("열기");
        app.click("확인");
        app.assert_text("accepted: 1 / rejected: 0");
    }
}
