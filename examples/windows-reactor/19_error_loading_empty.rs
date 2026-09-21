//! windows-reactor 예제 19 — 로딩/에러/빈 상태를 3분기하기
//!
//! **언제 쓰나**: 비동기 결과를 기다리는 화면.
//!
//! **모델**: 상태를 enum 하나로 만들고 `<Switch>`로 **빠짐없이** 분기한다.
//! `Result`를 그대로 쓰지 말고 UI가 구분해야 하는 상태
//! (`Idle/Loading/Ready/Empty/Failed`)를 명시한다 — "빈 목록"과 "아직 안 옴"은 다른 화면이다.
//!
//! **WinUI 쪽 대응**
//! - 로딩: `<Spinner>` → `ProgressRing(is_active: true)` — 애니메이션이 살아 있다.
//! - 진행률을 알면 `<Progress value={0..1}>` → `ProgressBar`(어댑터가 0..100으로 변환).
//! - 에러: `<Banner kind="error">` → `InfoBar(severity: Error)` + **재시도 버튼을 같은
//!   컨테이너에** 둔다(WinUI에는 토스트가 없으므로 화면 안에서 복구 동선을 제공한다).
//! - 빈 상태: 안내 문구 + "처음으로" 버튼.
//!
//! **주의**: `<Banner>`는 상태로 켜고 끈다 — `InfoBar`의 닫기(X)는 네이티브 동작이라
//! elm 상태를 모른다(예제 14의 주의 참고).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

#[derive(Clone, PartialEq, Debug)]
enum Phase {
    Idle,
    Loading,
    Ready(Vec<String>),
    Empty,
    Failed(String),
}

elm_magic::view! {
    fn ItemsView(phase: Phase = Phase::Idle) {
        <Col>
            <Switch on={phase}>
                <Case when={Phase::Idle}>
                    <Button on_click={phase = Phase::Loading}>"불러오기"</Button>
                </Case>
                <Case when={Phase::Loading}>
                    <Row>
                        <Spinner />
                        <Text>"불러오는 중…"</Text>
                    </Row>
                </Case>
                <Case when={Phase::Ready(items)}>
                    <Col>
                        <For each={items} as={it} key={it}>
                            <Text>"{it}"</Text>
                        </For>
                    </Col>
                </Case>
                <Case when={Phase::Empty}>
                    <Col>
                        <Text>"항목이 없습니다"</Text>
                        <Button on_click={phase = Phase::Idle}>"처음으로"</Button>
                    </Col>
                </Case>
                <Case when={Phase::Failed(msg)}>
                    <Col>
                        <Banner kind="error">{msg}</Banner>
                        <Button on_click={phase = Phase::Loading}>"다시 시도"</Button>
                    </Col>
                </Case>
            </Switch>
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
        context.window_title("elm-magic — 상태 3분기");
        View::component::<ElmView<ItemsView>>(ElmInput::new(ItemsViewProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    fn with(phase: Phase) -> (elm_magic_windows_reactor::PlanNode, elm_magic_windows_reactor::Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<ItemsView>(
            &mut ctx,
            &ItemsViewProps {
                phase: Some(phase),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    #[test]
    fn every_phase_renders_something_distinct() {
        let (_, idle) = with(Phase::Idle);
        assert!(idle.has_label("불러오기"));

        let (_, loading) = with(Phase::Loading);
        assert_eq!(loading.count("ProgressRing"), 1);
        assert!(loading.has_text("불러오는 중…"));

        let (_, ready) = with(Phase::Ready(vec!["a".to_string()]));
        assert!(ready.has_text("a"));

        let (_, empty) = with(Phase::Empty);
        assert!(empty.has_text("항목이 없습니다"));

        let (_, failed) = with(Phase::Failed("timeout".to_string()));
        assert_eq!(failed.count("InfoBar"), 1);
        // InfoBar 본문은 Banner 노드의 text로 실린다 (`Pass::texts`).
        assert!(failed.has_text("timeout"), "에러 본문이 계획에 실린다");
    }

    #[test]
    fn failure_offers_a_way_back() {
        let mut app = elm_magic::mount_with::<ItemsView>(ItemsViewProps {
            phase: Some(Phase::Failed("timeout".to_string())),
            ..Default::default()
        });
        app.click("다시 시도");
        app.assert_text("불러오는 중…");
    }
}
