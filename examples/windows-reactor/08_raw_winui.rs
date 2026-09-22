//! windows-reactor 예제 08 — `<Raw>` 탈출구: **WinUI 컨트롤을 돌려준다**
//!
//! **업스트림 대응**: `element-ref` · `drag-drop` · `pointer-*` · `tooltip-placement` —
//! 업스트림 샘플이 WinUI 컨트롤을 직접 잡는 자리와 **정확히 같은 지점**이다
//! (차이는 Reactor 타입을 elm 트리에 넣는 방법뿐).
//!
//! **언제 쓰나**: 이 어댑터가 표현하지 않는 WinUI 기능 — 간격/정렬 같은 레이아웃
//! 세부, `ScrollViewer`, `Grid`, `ListView`, `TabView`, 커스텀 컨트롤.
//!
//! **계약** (사양서 7.3)
//! - 이 어댑터에서 `<Raw>` 클로저는 **`&mut RawSlot`**(= `&mut Option<View>`)을 받는다.
//!   gpui 어댑터와 달리 **반환값이 버려지지 않는다** — 슬롯에 넣은 뷰가 그 자리에 들어간다.
//!   비워 두면 `View::empty()`가 들어간다.
//! - 헤드리스 테스트에서는 무시되고 `render_tree()`에 `[raw]`로만 보인다.
//! - 페이로드 타입이 다르면 런타임 panic("platform payload type mismatch")이 난다 —
//!   같은 트리를 다른 어댑터에 그대로 쓸 수 없다.
//!
//! **베스트 패턴**
//! - 스타일 계층이 없으므로 **WinUI 수준 조정은 전부 여기서** 한다: `spacing`,
//!   `padding`, `corner_radius`, `background`, `ThemeTransition` 등.
//! - Raw에 **elm 슬롯을 넘기지 않는다**(클로저는 `'static`). 값이 필요하면 캡처한다.
//! - 큰 WinUI 컴포넌트를 반복해서 쓰게 되면, 어댑터의 계획 층에 정식 매핑을 추가하는
//!   편이 낫다(`crates/elm-magic-windows-reactor/src/plan.rs`).

use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, ChildrenControl, Component, ComponentContext, LayoutControl, StackPanel,
    TextBlock, Thickness, View, ViewContext,
};

elm_magic::view! {
    fn Toolbar(count = 0) {
        <Col>
            // 어댑터는 `<Col>`을 spacing 0짜리 StackPanel로 만든다 — 간격이 필요하면 여기서.
            <Raw>|out: &mut RawSlot| {
                *out = Some(
                    StackPanel::new()
                        .spacing(12.0)
                        .children((
                            TextBlock::new().text("WinUI가 만든 텍스트"),
                            Border::new()
                                .border_thickness(Thickness::new(0.0, 1.0, 0.0, 0.0))
                                .height(1.0),
                        )),
                );
            }</Raw>
            "count: {count}"
            <Button on_click={count += 1}>"inc"</Button>
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
        context.window_title("elm-magic — Raw");
        View::component::<ElmView<Toolbar>>(ElmInput::new(ToolbarProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn raw_is_a_plan_node_with_the_closure() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Toolbar>(&mut ctx, &ToolbarProps::default());
        let (node, pass) = plan(&tree);

        let raw = node
            .children
            .iter()
            .find(|c| c.control() == "Raw")
            .expect("<Raw> 노드");
        assert!(raw.raw.is_some(), "클로저가 계획에 실려야 한다");
        assert_eq!(pass.count("Raw"), 1);
    }

    /// 페이로드 계약은 헤드리스에서도 확인할 수 있다(다른 타입을 넣으면 panic).
    #[test]
    fn raw_payload_type_must_match() {
        let tree = elm_magic::ui! { <Raw>|payload: &mut String| { payload.push_str("winui"); }</Raw> };
        let mut out = String::new();
        elm_magic::raw::invoke(&tree, &mut out);
        assert_eq!(out, "winui");
    }
}
