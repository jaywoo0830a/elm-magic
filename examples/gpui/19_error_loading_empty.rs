//! gpui 예제 19 — 로딩/에러/빈 상태를 gpui 화면에서 3분기하기
//!
//! **언제 쓰나**: 비동기 결과를 기다리는 화면. gpui에서는 `<Spinner>`가 gpui-kit
//! `Spinner`(애니메이션)로 그려지므로, 로딩 표시만 띄워 두면 "살아 있다"는 신호가 된다.
//!
//! **모델**: 상태를 enum 하나로 만들고 `<Switch>`로 **빠짐없이** 분기한다.
//! `Result`를 그대로 쓰지 말고 UI가 구분해야 하는 상태(`Idle/Loading/Ready/Empty/Failed`)를
//! 명시한다 — "빈 목록"과 "아직 안 옴"은 다른 화면이다.
//!
//! **gpui 특이점**
//! - 로딩은 `Spinner` + 텍스트를 `Row`로 묶어 `align: center`를 준다(가운데 정렬).
//! - 에러는 `Banner kind="error"` + 재시도 버튼을 **같은 `Col`에** 둔다 —
//!   gpui에는 토스트가 없으므로 화면 안에서 복구 동선을 제공하는 편이 낫다.
//! - 성공/실패 애니메이션을 넣고 싶으면 `<Raw>`로 gpui-kit 애니메이션을 얹는다.
//!
//! **주의**: `<-`의 결과는 슬롯에 들어간다 — 실패를 표현하려면 슬롯 타입을
//! `Result<_, String>`로 두고 `<-`가 그 값을 돌려주게 한다(예제 11).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

#[derive(Clone, PartialEq, Debug)]
enum Phase {
    Idle,
    Loading,
    Ready(Vec<String>),
    Empty,
    Failed(String),
}

elm_magic::css! {
    .center { align: center; justify: center; gap: 8; padding: 24; }
    .error { color: error; }
}

elm_magic::view! {
    fn ItemsView(phase: Phase = Phase::Idle) {
        <Col class="center">
            <Switch on={phase}>
                <Case when={Phase::Idle}>
                    <Button on_click={phase = Phase::Loading}>"불러오기"</Button>
                </Case>
                <Case when={Phase::Loading}>
                    <Row class="center">
                        <Spinner />
                        "불러오는 중…"
                    </Row>
                </Case>
                <Case when={Phase::Ready(items)}>
                    <Col>
                        <For each={items} as={it} key={it}>
                            <Row>"{it}"</Row>
                        </For>
                    </Col>
                </Case>
                <Case when={Phase::Empty}>
                    <Col class="center">
                        <Text class="muted">"항목이 없습니다"</Text>
                        <Button on_click={phase = Phase::Idle}>"처음으로"</Button>
                    </Col>
                </Case>
                <Case when={Phase::Failed(msg)}>
                    <Col class="center">
                        <Banner kind="error">"{msg}"</Banner>
                        <Button on_click={phase = Phase::Loading}>"다시 시도"</Button>
                    </Col>
                </Case>
            </Switch>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(cx.new(ElmView::<ItemsView>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(phase: Phase) -> elm_magic::testing::TestApp<ItemsView> {
        elm_magic::mount_with::<ItemsView>(ItemsViewProps {
            phase: Some(phase),
            ..Default::default()
        })
    }

    #[test]
    fn every_phase_renders_something() {
        with(Phase::Idle).assert_text("불러오기");
        with(Phase::Loading).assert_text("[spinner]");
        with(Phase::Ready(vec!["a".into()])).assert_text("a");
        with(Phase::Empty).assert_text("항목이 없습니다");
        with(Phase::Failed("timeout".into())).assert_text("timeout");
    }

    #[test]
    fn failure_offers_a_way_back() {
        let mut app = with(Phase::Failed("timeout".into()));
        app.click("다시 시도");
        app.assert_text("[spinner]");
    }
}