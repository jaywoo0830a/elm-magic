//! windows-reactor 예제 17 — 탭/표 헤더가 어떻게 옮겨지는가 (+ 진짜 `TabView`는 `<Raw>`)
//!
//! **elm 태그의 매핑** (`crates/elm-magic-windows-reactor/src/plan.rs`)
//! | elm | WinUI | 비고 |
//! |---|---|---|
//! | `<Tab active on_click>` | `Button` | `active`는 계획에만 남는다(스타일 계층이 없으므로) |
//! | `<Th on_click>` | `Button` | 정렬 가능한 헤더 |
//! | `<Th>` (클릭 없음) | `TextBlock` | 굵은 글씨 |
//! | `<Td>` | `TextBlock` | |
//!
//! **왜 `TabView`가 아닌가**: Reactor의 `TabView`는 **컨테이너**다 —
//! `TabViewItem`을 **슬롯**으로 받는다(`TabViewSlot::TabItems`). elm의 형제 `<Tab>`들은
//! 컨테이너 없이 나열되므로 그대로 옮길 수 없다. 그래서 버튼으로 옮기고,
//! **진짜 탭 UI가 필요하면 `<Raw>`에서 `TabView`를 만든다**(아래 `tabs`).
//!
//! **`<Tab>`의 활성 표시**: `active`는 계획에 실리지만 스타일 계층이 없어 색이 바뀌지
//! 않는다. WinUI에서 활성 표시를 하려면 `<Raw>` + `TabView::selected_index`를 쓰거나,
//! 버튼 라벨/본문으로 상태를 드러낸다(접근성에도 낫다).
//!
//! **표**: elm의 `<Th>`/`<Td>`는 **텍스트**로 옮겨진다 — 2D 표가 필요하면 `<Raw>`로
//! `Grid`(+ 행/열 정의)나 `ListView`를 쓰고, **정렬/선택 상태는 elm 슬롯**에 둔다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, ChildrenControl, Component, ComponentContext, SlotsControl, StackPanel, TabView,
    TabViewItem, TabViewSlot, TextBlock, View, ViewContext,
};

elm_magic::view! {
    fn Tabs(tab = 0, sort = String::new()) {
        <Col>
            <Row>
                <Tab active={tab == 0} on_click={tab = 0}>"Home"</Tab>
                <Tab active={tab == 1} on_click={tab = 1}>"Stats"</Tab>
            </Row>
            "tab: {tab}"

            <Row>
                <Th on_click={sort = String::from("name")}>"Name"</Th>
                <Th>"Age"</Th>
                <Td>"alice"</Td>
                <Td>"30"</Td>
            </Row>
            "sort: {sort}"

            // 진짜 WinUI TabView가 필요하면 Raw가 정식 통로다.
            <Raw>|out: &mut RawSlot| {
                let items = StackPanel::new().spacing(4.0).children((
                    TabViewItem::new()
                        .header("Home")
                        .tag("home")
                        .content(TextBlock::new().text("home content")),
                    TabViewItem::new()
                        .header("Stats")
                        .tag("stats")
                        .content(TextBlock::new().text("stats content")),
                ));
                *out = Some(TabView::new().selected_index(0usize).slot(TabViewSlot::TabItems, items));
            }</Raw>
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
        context.window_title("elm-magic — 탭과 표");
        View::component::<ElmView<Tabs>>(ElmInput::new(TabsProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn tabs_and_headers_become_buttons() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Tabs>(&mut ctx, &TabsProps::default());
        let (node, pass) = plan(&tree);

        // Tab 둘 + 클릭 가능한 Th 하나 = 버튼 3개.
        assert_eq!(pass.count("Button"), 3);
        assert_eq!(pass.labeled, vec!["Home", "Stats", "Name"]);

        // 정적 Th와 Td는 텍스트(헤더는 굵게).
        assert!(pass.has_text("Age") && pass.has_text("alice") && pass.has_text("30"));
        assert!(matches!(
            node.children[1].children[1].kind,
            elm_magic_windows_reactor::PlanKind::Text { strong: true }
        ));
    }

    #[test]
    fn tab_state_is_kept_in_the_plan() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Tabs>(&mut ctx, &TabsProps::default());
        let (node, _) = plan(&tree);

        let tabs = &node.children[0];
        assert!(tabs.children[0].active, "Home이 활성");
        assert!(!tabs.children[1].active);
    }

    #[test]
    fn clicking_a_header_records_the_sort_key() {
        let mut app = elm_magic::mount!(Tabs);
        app.click("Name");
        app.assert_text("sort: name");
    }
}
