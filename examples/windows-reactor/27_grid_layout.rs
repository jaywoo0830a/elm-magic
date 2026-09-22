//! windows-reactor 예제 27 — **Grid 레이아웃**: 2D 배치 · 고정/가변 트랙 · 랩 그리드
//!
//! **언제 쓰나**: `StackPanel`(1D)로 안 되는 배치 — "사이드바 + 본문", "헤더/본문/푸터".
//!
//! **Grid는 세 가지를 정한다**
//! 1. **트랙**: `rows([..])` / `columns([..])` — `GridLength::{Auto, Pixel(d), Star(w)}`.
//!    `Auto`=내용만큼, `Pixel`=고정, `Star`=남는 공간을 비율로(`GridLength::STAR` = `1*`).
//! 2. **간격**: `row_spacing` / `column_spacing` — 예제 23의 리듬 값을 그대로 쓴다.
//! 3. **자식의 자리**: `GridChildExt::{grid_row, grid_column, grid_row_span,
//!    grid_column_span}` — **자식 쪽에** 준다(부모가 아니라). `Border`/`TextBlock` 등
//!    모든 레이아웃 컨트롤에 붙는다.
//!
//! **`<Raw>`에서만 쓸 수 있다** — elm 트리(`<Col>`/`<Row>`)는 1D `StackPanel`로 매핑된다
//! (`plan.rs`의 매핑표). 그래서 "껍데기는 Grid(Raw) + 내용은 elm 컴포넌트"가 자연스럽다.
//!
//! **랩(wrap) 배치**: `VariableSizedWrapGrid`는 "고정 크기 타일을 자동 줄바꿈"으로 배치한다
//! (`item_width`/`item_height`/`orientation`) — 반응형 카드 격자에 쓴다. 트랙을 미리
//! 정하지 않아도 되므로 **개수가 변하는 카드 목록**에 Grid보다 낫다.
//!
//! **주의**
//! - 트랙 수를 넘는 `grid_row`/`grid_column`은 **조용히 무시**된다(예외가 아니다).
//!   그래서 트랙 정의를 [`row_spec`]/[`column_spec`] **한 곳**에 두고 자식 배치와 함께 본다.
//! - `Grid::background`는 표면 색만 준다 — 카드처럼 테두리/여백이 필요하면 `Border`로
//!   감싸는 편이 낫다(Grid에는 `padding`/`corner_radius`가 없다).
//! - 자식의 `margin`은 Grid에서도 유효하다 — 여백이 필요하면 자식에 준다(예제 23).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Component, ComponentContext, ContentControl, CornerRadius,
    Grid, GridChildExt, GridLength, KeyedView, LayoutControl, Orientation, TextBlock, ThemeBrush,
    Thickness, VariableSizedWrapGrid, View, ViewContext,
};

/// 3행: 헤더(고정) / 본문(남는 공간) / 상태바(고정).
fn row_spec() -> [GridLength; 3] {
    [
        GridLength::Pixel(48.0),
        GridLength::STAR,
        GridLength::Pixel(28.0),
    ]
}

/// 2열: 사이드바(고정 폭) / 본문(남는 공간).
fn column_spec() -> [GridLength; 2] {
    [GridLength::Pixel(200.0), GridLength::STAR]
}

/// 트랙 정의와 자식 배치를 **한 곳에서** 본다 — 어긋나면 조용히 무시되므로.
///
/// 표면 하나 = 배치(`row`/`column`/`span`) + 스타일 + 내용. Grid에서는 **자리도 스타일**이라
/// 같은 함수가 셋을 함께 받는다.
fn surface(text: &str, row: i32, column: i32, span: i32) -> View {
    Border::new()
        // 배치는 **자식**에게 준다(`View`에는 없고 컨트롤에만 있다).
        .grid_row(row)
        .grid_column(column)
        .grid_column_span(span)
        .background(Brush::from(ThemeBrush::CardBackground))
        .border_brush(Brush::from(ThemeBrush::CardStroke))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(4.0))
        .padding(Thickness::uniform(8.0))
        .content(TextBlock::new().text(text).font_size(12.0))
}

fn shell_grid() -> View {
    Grid::new()
        .rows(row_spec())
        .columns(column_spec())
        .row_spacing(8.0)
        .column_spacing(8.0)
        .background(Brush::from(ThemeBrush::SolidBackground))
        .children((
            // 헤더: 0행, 두 열을 가로지른다.
            surface("헤더 — grid_row(0) + grid_column_span(2)", 0, 0, 2),
            // 사이드바: 1행 0열.
            surface("사이드바 — Pixel(200)", 1, 0, 1),
            // 본문: 1행 1열 — 남는 공간(Star)을 차지한다.
            surface("본문 — Star", 1, 1, 1),
            // 상태바: 2행, 두 열.
            surface("상태바 — Pixel(28)", 2, 0, 2),
        ))
}

/// 랩 그리드 — 고정 크기 타일이 자동으로 줄바꿈된다(개수가 변하는 카드 목록에 좋다).
fn wrap_tiles() -> View {
    let tiles: Vec<KeyedView> = (1..=8)
        .map(|index| {
            KeyedView::new(
                index as u64,
                Border::new()
                    .width(120.0)
                    .height(56.0)
                    .background(Brush::from(ThemeBrush::CardBackground))
                    .border_brush(Brush::from(ThemeBrush::CardStroke))
                    .border_thickness(Thickness::uniform(1.0))
                    .corner_radius(CornerRadius::uniform(6.0))
                    .padding(Thickness::uniform(8.0))
                    .content(
                        TextBlock::new()
                            .text(format!("타일 {index}"))
                            .font_size(11.0),
                    ),
            )
        })
        .collect();

    VariableSizedWrapGrid::new()
        .item_width(132.0)
        .item_height(64.0)
        .orientation(Orientation::Horizontal)
        .keyed_children(tiles)
}

elm_magic::view! {
    fn Dashboard() {
        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(shell_grid());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(wrap_tiles());
            }</Raw>
            <Text>"Grid 3행 × 2열 · 랩 타일 8개"</Text>
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
        context.window_title("elm-magic — Grid 레이아웃");
        View::component::<ElmView<Dashboard>>(ElmInput::new(DashboardProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    /// 트랙 값 꺼내기 — `GridLength`에는 `PartialEq`가 없어서 match로 읽는다.
    fn pixel(length: GridLength) -> Option<f64> {
        match length {
            GridLength::Pixel(value) => Some(value),
            _ => None,
        }
    }

    fn is_star(length: GridLength) -> bool {
        matches!(length, GridLength::Star(_))
    }

    /// 헤더/상태바는 고정 높이, 본문만 남는 공간을 쓴다.
    #[test]
    fn the_row_spec_pins_the_bars_and_frees_the_body() {
        let rows = row_spec();
        assert_eq!(rows.len(), 3);
        assert_eq!(pixel(rows[0]), Some(48.0), "헤더는 고정");
        assert!(is_star(rows[1]), "본문은 남는 공간");
        assert_eq!(pixel(rows[2]), Some(28.0), "상태바는 고정");
        assert!(
            pixel(rows[2]).unwrap() < pixel(rows[0]).unwrap(),
            "상태바가 헤더보다 낮다"
        );
    }

    /// 사이드바는 고정 폭, 본문이 남는 공간을 가져간다.
    #[test]
    fn the_column_spec_gives_the_body_the_rest() {
        let columns = column_spec();
        assert_eq!(pixel(columns[0]), Some(200.0), "사이드바 고정");
        assert!(is_star(columns[1]), "본문이 남는 공간");
    }

    /// `GridLength::STAR`는 `1*`다(관례를 코드로 확인한다).
    #[test]
    fn star_is_one_star() {
        match GridLength::STAR {
            GridLength::Star(weight) => assert_eq!(weight, 1.0),
            other => panic!("STAR는 Star(1.0)이어야 한다: {other:?}"),
        }
    }

    /// 구조: Grid 껍데기와 랩 타일 두 묶음은 Raw가 만든다.
    #[test]
    fn the_plan_has_two_raw_blocks() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Dashboard>(&mut ctx, &DashboardProps::default());
        let (node, pass) = plan(&tree);
        assert_eq!(pass.count("Raw"), 2);
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(kinds, vec!["Raw", "Raw", "TextBlock"]);
    }
}
