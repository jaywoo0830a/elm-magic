//! windows-reactor 예제 06 — 제어 흐름 태그 (`<If>` / `<For>` / `<Switch>` / `<>`)
//!
//! **언제 쓰나**: 본문에서 조건/반복을 태그로 쓸 때. **어댑터가 바뀌지 않는 이유**:
//! 제어 흐름 태그는 매크로가 **컴파일타임에 펼치는** 문법이라 트리에는 감싼 요소만
//! 남는다(감싸는 컨트롤이 생기지 않는다).
//!
//! **계약** (`tests/0.8/*.rs`)
//! - `<If when={..}>` + `<Else>` — 둘 다 레이아웃 노드를 만들지 않는다.
//! - `<For each={..} as={t} key={t.id}>` — key가 있으면 keyed 슬롯(자식 상태 보존).
//! - `<Switch on={..}>` + `<Case when={패턴}>` + `<Default>` — 패턴 바인딩 허용.
//! - `<> … </>` — 프래그먼트. **자식 위치**에서는 매크로가 펼치고(노드 없음),
//!   **본문 전체**가 `<>`면 `Fragment` 노드가 남는다(계획도 두 경우를 구분한다).
//!
//! **Reactor 쪽 결과**
//! - `<If>`/`<Switch>`는 **분기마다 다른 컨트롤**을 낳는다 — WinUI는 Reactor가
//!   타입별로 맞는 컨트롤을 만들고 지운다.
//! - `<For>`는 부모 `StackPanel`의 자식으로 **평평하게** 들어간다(래퍼 컨트롤 없음).
//!   자식 목록은 위치 기반 키로 넘어간다(예제 07의 한계 참고).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

#[derive(Clone, PartialEq, Debug)]
enum Status {
    Idle,
    Loading,
    Failed(String),
}

elm_magic::view! {
    fn Panel(
        items: Vec<String> = vec![],
        status: Status = Status::Idle,
        show_hint = true,
    ) {
        <Col>
            <If when={show_hint}>
                <Text>"힌트"</Text>
            </If>

            <For each={items} as={t} key={t}>
                <Text>"{t}"</Text>
            </For>

            <Switch on={status}>
                <Case when={Status::Idle}><Text>"대기"</Text></Case>
                <Case when={Status::Loading}><Spinner /></Case>
                <Case when={Status::Failed(msg)}><Banner kind="error">{msg}</Banner></Case>
                <Default><Text>"—"</Text></Default>
            </Switch>

            <>
                <Divider />
                <Text>"끝"</Text>
            </>
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
        context.window_title("elm-magic — 제어 흐름");
        View::component::<ElmView<Panel>>(ElmInput::new(PanelProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    fn panel(props: PanelProps) -> (elm_magic_windows_reactor::PlanNode, elm_magic_windows_reactor::Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Panel>(&mut ctx, &props);
        plan(&tree)
    }

    #[test]
    fn branches_choose_different_controls() {
        // Loading → ProgressRing, Failed → InfoBar.
        let (node, pass) = panel(PanelProps {
            status: Some(Status::Loading),
            ..Default::default()
        });
        assert_eq!(pass.count("ProgressRing"), 1);

        let (node, _) = panel(PanelProps {
            status: Some(Status::Failed("timeout".to_string())),
            ..Default::default()
        });
        let banner = node
            .children
            .iter()
            .find(|c| c.control() == "InfoBar")
            .expect("InfoBar");
        assert_eq!(banner.text, "timeout");
    }

    #[test]
    fn for_children_are_flattened_into_the_parent_stack() {
        let (node, pass) = panel(PanelProps {
            items: Some(vec!["a".to_string(), "b".to_string()]),
            ..Default::default()
        });
        // 래퍼 컨트롤 없이 자식이 늘어난다 (StackPanel은 루트 하나뿐).
        assert_eq!(pass.count("StackPanel"), 1);
        assert!(pass.has_text("a") && pass.has_text("b"));
        assert!(node.children.iter().any(|c| c.text == "a"));
    }

    #[test]
    fn fragment_children_keep_their_order() {
        let (node, _) = panel(PanelProps::default());
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        // 힌트 / 대기 / Divider / 끝 — `<>`는 펼쳐져 Divider와 텍스트가 이어진다.
        assert_eq!(kinds, vec!["TextBlock", "TextBlock", "Border", "TextBlock"]);
    }
}
