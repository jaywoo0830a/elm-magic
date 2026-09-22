//! windows-reactor 예제 28 — **스크롤과 고정 영역**: `ScrollViewer` · 머리글/바닥글 고정
//!
//! **언제 쓰나**: 내용이 창보다 길 때. 스크롤은 **WinUI의 몫**이고, 우리는 "무엇이 고정되고
//! 무엇이 스크롤되는가"만 정한다.
//!
//! **구성**
//! - 고정 띠 = `Border::height(..)` + 배경 + 한쪽 테두리(예제 23/24).
//! - 스크롤 영역 = `ScrollViewer::new().content(..)` +
//!   `vertical_scroll_bar_visibility(ScrollBarVisibility::{Auto, Visible, Hidden, Disabled})`.
//!   `Auto`가 기본 감각이다(내용이 넘칠 때만 막대가 보인다).
//! - 스크롤 영역은 **남는 공간을 차지해야** 한다 — `StackPanel`은 자식을 늘리지 않으므로
//!   크기를 명시하거나(예제 27의 `Grid` `Star` 행) 부모가 늘려 주어야 한다.
//!
//! **목록을 `<Raw>`에서 만든다** — elm 트리(`<For>`)는 Raw 안에 넣을 수 없다. 그래서 렌더
//! 본문에서 `let list = rows.clone();`으로 **값을 캡처**하고, 클로저가 목록을 만든다.
//! 길이가 변하는 목록은 `keyed_children` + `KeyedView`(키는 인덱스)로 만든다(예제 22).
//!
//! **주의**
//! - 이 방식은 **렌더마다 목록 전체를 다시 만든다** — 항목이 수천 개면 elm 쪽 `<For>`로
//!   그리는 편이 낫다(어댑터가 위치 기반 키로 넘긴다).
//! - **스크롤 위치는 elm 상태가 아니다**(WinUI가 소유한다). elm이 다시 그려도 유지된다 —
//!   "상태는 elm, 표현은 WinUI"의 경계가 여기서도 그대로다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Component, ComponentContext, ContentControl, CornerRadius,
    KeyedView, LayoutControl, ScrollBarVisibility, ScrollViewer, StackPanel, TextBlock, ThemeBrush,
    Thickness, VerticalAlignment, View, ViewContext,
};

/// 고정 띠 — 머리글/바닥글. 한쪽에만 1px 선을 그어 경계를 만든다.
fn bar(text: &str, height: f64, line_on_top: bool) -> View {
    let line = if line_on_top {
        Thickness::new(0.0, 1.0, 0.0, 0.0)
    } else {
        Thickness::new(0.0, 0.0, 0.0, 1.0)
    };

    Border::new()
        .height(height)
        .background(Brush::from(ThemeBrush::SolidBackground))
        .border_brush(Brush::from(ThemeBrush::CardStroke))
        .border_thickness(line)
        .padding(Thickness::new(12.0, 0.0, 12.0, 0.0))
        .content(
            TextBlock::new()
                .text(text)
                .font_size(12.0)
                .vertical_alignment(VerticalAlignment::Center),
        )
}

/// 스크롤 영역 — 목록이 길면 막대가 나타난다(`Auto`).
fn list_view(rows: Vec<String>) -> View {
    let items: Vec<KeyedView> = rows
        .iter()
        .enumerate()
        .map(|(index, text)| {
            KeyedView::new(
                index as u64,
                Border::new()
                    .background(Brush::from(ThemeBrush::CardBackground))
                    .border_brush(Brush::from(ThemeBrush::CardStroke))
                    .border_thickness(Thickness::uniform(1.0))
                    .corner_radius(CornerRadius::uniform(4.0))
                    .padding(Thickness::uniform(8.0))
                    .content(TextBlock::new().text(text.clone()).font_size(12.0)),
            )
        })
        .collect();

    // 빈 목록과 채워진 목록은 **다른 모양**이다 — 스크롤 영역 자체는 그대로 둔다.
    let body = if items.is_empty() {
        StackPanel::new().spacing(4.0).children((TextBlock::new()
            .text("목록이 비어 있다 — 아래 버튼으로 행을 추가한다")
            .font_size(12.0)
            .opacity(0.7),))
    } else {
        StackPanel::new().spacing(6.0).keyed_children(items)
    };

    ScrollViewer::new()
        .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
        .content(body)
}

elm_magic::view! {
    fn Scrolling(rows: Vec<String> = vec![]) {
        // `<Raw>`가 캡처할 목록은 렌더 본문에서 만든다(값이므로 복제해 넘긴다).
        let list = rows.clone();

        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(bar("머리글 — 스크롤되지 않는다", 44.0, false));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(list_view(list.clone()));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(bar("바닥글 — 스크롤되지 않는다", 32.0, true));
            }</Raw>

            <If when={rows.len() > 0}>
                <Text>"목록 있음 — 스크롤해 보세요"</Text>
            <Else>
                <Text>"목록 없음 — 버튼으로 행을 추가하세요"</Text>
            </Else>
            </If>

            <Button on_click={rows.push(format!("행 {}", rows.len() + 1))}>"행 추가"</Button>
            <Button on_click={rows.push(format!("행 {}", rows.len() + 1));
                              rows.push(format!("행 {}", rows.len() + 1));
                              rows.push(format!("행 {}", rows.len() + 1));
                              rows.push(format!("행 {}", rows.len() + 1));
                              rows.push(format!("행 {}", rows.len() + 1))}>"5개 추가"</Button>
            <Button on_click={rows.clear()}>"비우기"</Button>
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
        context.window_title("elm-magic — 스크롤과 고정 영역");
        View::component::<ElmView<Scrolling>>(ElmInput::new(ScrollingProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    fn plan_for(rows: Vec<String>) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Scrolling>(
            &mut ctx,
            &ScrollingProps {
                rows: Some(rows),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 구조: 머리글/목록/바닥글 세 묶음은 Raw, 라벨과 버튼 셋은 elm이 그린다.
    #[test]
    fn the_plan_has_three_raw_blocks() {
        let (node, pass) = plan_for(Vec::new());
        assert_eq!(pass.count("Raw"), 3);
        assert_eq!(pass.count("Button"), 3);
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(
            kinds,
            vec![
                "Raw",
                "Raw",
                "Raw",
                "TextBlock",
                "Button",
                "Button",
                "Button"
            ]
        );
    }

    /// 목록의 유무가 다른 문구를 낸다(빈 화면도 정상 상태다).
    #[test]
    fn the_label_follows_the_list() {
        assert!(plan_for(Vec::new())
            .1
            .has_text("목록 없음 — 버튼으로 행을 추가하세요"));
        assert!(plan_for(vec!["행 1".to_string()])
            .1
            .has_text("목록 있음 — 스크롤해 보세요"));
    }

    /// 버튼이 elm 상태를 바꾼다(헤드리스에서 클릭까지 확인).
    #[test]
    fn adding_and_clearing_rows_flips_the_label() {
        let mut app = elm_magic::mount!(Scrolling);
        app.assert_text("목록 없음 — 버튼으로 행을 추가하세요");
        app.click("행 추가");
        app.assert_text("목록 있음 — 스크롤해 보세요");
        app.click("비우기");
        app.assert_text("목록 없음 — 버튼으로 행을 추가하세요");
    }

    /// 한 번에 여러 행을 넣는 버튼도 같은 규칙(문장 여러 개)으로 동작한다.
    #[test]
    fn the_bulk_button_pushes_five_rows() {
        let mut app = elm_magic::mount!(Scrolling);
        app.click("5개 추가");
        app.assert_text("목록 있음 — 스크롤해 보세요");
    }
}
