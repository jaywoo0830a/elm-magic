//! windows-reactor 예제 04 — 이벤트가 메시지가 되어 상태를 바꾼다
//!
//! **업스트림 대응**: `function-component` · `radio-buttons` — 업스트림은 `enum Message`와
//! `Component::update`를 손으로 쓰고, elm에서는 그 enum을 매크로가 만든다. "핸들러는
//! 항상 메시지 큐를 거친다"는 규칙도 같다.
//!
//! **언제 쓰나**: "클릭했는데 화면이 안 바뀐다"를 이해하거나, 갱신 경로를 줄이고 싶을 때.
//!
//! **동작 순서** (`crates/elm-magic-windows-reactor/src/winui.rs`)
//! 1. `ElmView::view()`가 `elm_magic::frame::<C>(ctx, props)`로 트리를 만들고
//!    `plan()`으로 계획을 만든 뒤 WinUI 컨트롤로 옮긴다.
//! 2. 버튼/입력/체크의 핸들러는 `context.callback(..)`으로 감싸져
//!    [`ElmMessage`](elm_magic_windows_reactor::ElmMessage)로 **메시지 큐**에 들어간다.
//! 3. Reactor가 그 메시지를 `Component::update`로 배달하면, 어댑터가 elm 핸들러를
//!    아레나에 대해 실행하고 곧바로 `drive::run`으로 효과/스트림을 구동한다.
//! 4. 상태가 바뀌었으므로 Reactor가 **그 컴포넌트를 다시 발행(publish)** 한다 →
//!    `view()`가 다시 돌고 WinUI 트리가 갱신된다.
//!
//! **중요**: Reactor는 WinRT 콜백 안에서 `update`/재조정을 돌리지 않는다(문서의
//! "이벤트 FIFO"). 그래서 핸들러는 항상 **메시지 큐를 거쳐** 실행된다 — elm의
//! "이벤트 = 할당" 모델과 정확히 맞는다.
//!
//! **베스트 패턴 (렌더 줄이기)**
//! - 자주 바뀌는 값(입력 중 텍스트)은 **작은 `ElmView`로 분리**한다. 큰 화면 하나에
//!   넣으면 타이핑마다 전체가 다시 만들어진다.
//! - 무거운 파생 계산은 슬롯이 아니라 효과(`<-`)나 앱 코드에서 한다.

use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    fn Counter(n = 0) {
        <Col>
            "Count: {n}"
            <Button on_click={n += 1}>"+"</Button>
        </Col>
    }
}

elm_magic::view! {
    fn Draft(value = String::new()) {
        <Col>
            <Input value={value.clone()} on_change={value = _} />
            "draft: {value}"
        </Col>
    }
}

/// 화면 셸 — 여기서 바뀌는 것은 없다(자식 둘은 각자 상태를 갖는다).
struct Shell;

impl Component for Shell {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 메시지 루프");
        View::fragment((
            View::component::<ElmView<Counter>>(ElmInput::new(CounterProps::default())),
            View::component::<ElmView<Draft>>(ElmInput::new(DraftProps::default())),
        ))
    }
}

fn main() {
    windows_reactor::App::run_component::<Shell>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::plan;

    /// 상태 계약은 헤드리스로 — 어댑터를 띄우지 않고 가장 빠르게 확인한다.
    #[test]
    fn state_contract_is_platform_independent() {
        let mut app = elm_magic::mount!(Counter);
        app.assert_text("Count: 0");
        app.click("+");
        app.assert_text("Count: 1");
    }

    /// 어댑터가 "무엇을 그릴 것인가"로 바꾼 결과는 계획에서 확인한다.
    #[test]
    fn the_adapter_plans_two_children() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Draft>(&mut ctx, &DraftProps::default());
        let (node, pass) = plan(&tree);

        assert_eq!(node.control(), "StackPanel");
        assert_eq!(pass.inputs, 1);
        assert_eq!(pass.count("TextBox"), 1);
    }
}
