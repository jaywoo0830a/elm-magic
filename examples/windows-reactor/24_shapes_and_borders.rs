//! windows-reactor 예제 24 — **모양과 테두리**: 반지름 · 변별 테두리 · 도형
//! **업스트림 대응**: `shape` · `card` · `icon`(아바타/점 표시) — 업스트림은
//! `Ellipse`/`Rectangle`/`Line`을 그대로 쓴다. 이 예제도 같은 타입을 `<Raw>`에서 쓴다
//! (`CornerRadius`는 `Border`의 속성이다).
//!
//!
//! **언제 쓰나**: 카드/배지/아바타처럼 "형태"가 정보를 전달할 때.
//!
//! **`CornerRadius` 두 형태**
//! - `CornerRadius::uniform(r)` — 사방 같은 값. 0=각짐, 4=컨트롤, 8=카드,
//!   **999=알약(pill)**. 알약은 "높이보다 큰 값"이라는 관례다([`is_pill`]).
//! - `CornerRadius::new(tl, tr, br, bl)` — 변마다 다르게. "위만 둥근 헤더"가 이걸 쓴다.
//!
//! **테두리는 변마다 줄 수 있다**
//! - `Border::border_thickness(Thickness::new(4, 0, 0, 0))` = 왼쪽 4px 액센트 바
//!   (목록에서 "선택된 항목" 표시에 쓴다 — 배경색을 바꾸는 것보다 조용하다).
//! - `border_brush`는 **선**의 색, `background`는 **면**의 색. 둘 다 `Brush`다.
//! - 배경이 투명하면(=`background` 없이 `border_brush`만) 반지름은 **선에만** 적용된다.
//!
//! **도형은 WinUI `Shape`를 쓴다** — Reactor가 `Ellipse`/`Rectangle`/`Line`을 노출한다.
//! - 아바타·점 표시 = `Ellipse`(`fill`/`stroke`/`stroke_thickness`).
//! - 둥근 막대 = `Rectangle`(`radius_x`/`radius_y`).
//! - 긴 구분선 = `Line`(`x1..y2`). 짧은 구분선은 1px `Border`가 더 간단하다(예제 23).
//! - 도형도 **레이아웃 컨트롤**이다 — 크기는 `LayoutControl::{width, height}`로 준다.

use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Color, Component, ComponentContext, ContentControl,
    CornerRadius, Ellipse, KeyedView, LayoutControl, Line, Orientation, Rectangle, StackPanel,
    TextBlock, ThemeBrush, Thickness, VerticalAlignment, View, ViewContext,
};

/// 반지름 스케일 — 0(각짐)부터 알약까지.
const RADII: [(f64, &str); 5] = [
    (0.0, "0 — 각짐"),
    (4.0, "4 — 컨트롤"),
    (8.0, "8 — 카드"),
    (16.0, "16 — 큰 표면"),
    (999.0, "999 — 알약(pill)"),
];

/// 알약 판정 — 999는 "높이보다 큰 값"이라는 관례다. **값을 외우지 않고 함수로** 둔다.
/// 화면은 라벨을 그대로 쓰고, 이 술어는 `#[cfg(test)]`가 스케일 전체에 대해
/// 단언한다 — `allow`는 "테스트 전용 계약"이라는 표시다.
#[allow(dead_code)]
fn is_pill(radius: f64) -> bool {
    radius >= 100.0
}

/// 반지름 스와치 — 같은 사각형, 다른 모서리.
fn radius_swatches() -> View {
    let rows: Vec<KeyedView> = RADII
        .iter()
        .enumerate()
        .map(|(index, (radius, label))| {
            KeyedView::new(
                index as u64,
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(12.0)
                    .children((
                        Border::new()
                            .width(56.0)
                            .height(40.0)
                            .background(Brush::from(ThemeBrush::Accent))
                            .corner_radius(CornerRadius::uniform(*radius)),
                        TextBlock::new()
                            .text(*label)
                            .font_size(12.0)
                            .vertical_alignment(VerticalAlignment::Center),
                    )),
            )
        })
        .collect();
    StackPanel::new().spacing(10.0).keyed_children(rows)
}

/// 변마다 다른 모서리 — "위만 둥근" 헤더 표면.
fn per_corner_card() -> View {
    Border::new()
        .background(Brush::from(ThemeBrush::CardBackground))
        .border_brush(Brush::from(ThemeBrush::CardStroke))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::new(12.0, 12.0, 0.0, 0.0))
        .padding(Thickness::uniform(16.0))
        .content(
            TextBlock::new()
                .text("CornerRadius::new(12, 12, 0, 0) — 위만 둥글다")
                .font_size(12.0),
        )
}

/// 변별 테두리 — 왼쪽 4px 액센트 바.
fn accent_bar_card() -> View {
    Border::new()
        .background(Brush::from(ThemeBrush::CardBackground))
        .border_brush(Brush::from(ThemeBrush::Accent))
        .border_thickness(Thickness::new(4.0, 0.0, 0.0, 0.0))
        .padding(Thickness::new(12.0, 8.0, 12.0, 8.0))
        .content(
            TextBlock::new()
                .text("border_thickness::new(4, 0, 0, 0) — 선택 표시")
                .font_size(12.0),
        )
}

/// 도형 — 아바타(원) / 점 / 둥근 막대 / 긴 구분선.
fn shapes() -> View {
    StackPanel::new().spacing(14.0).children((
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(12.0)
            .children((
                // 아바타: 원 + 테두리.
                Ellipse::new()
                    .width(40.0)
                    .height(40.0)
                    .fill(Brush::from(ThemeBrush::Accent))
                    .stroke(Brush::from(ThemeBrush::CardStroke))
                    .stroke_thickness(2.0),
                // 점 표시(작은 원) — 글자 옆에 두려면 가운데 정렬을 준다.
                Ellipse::new()
                    .width(8.0)
                    .height(8.0)
                    .fill(Brush::from(ThemeBrush::SystemCritical))
                    .vertical_alignment(VerticalAlignment::Center),
                // 둥근 막대: Rectangle + radius.
                Rectangle::new()
                    .width(120.0)
                    .height(10.0)
                    .radius_x(5.0)
                    .radius_y(5.0)
                    .fill(Brush::from(ThemeBrush::Accent)),
                TextBlock::new()
                    .text("Ellipse / Ellipse / Rectangle")
                    .font_size(11.0)
                    .opacity(0.6)
                    .vertical_alignment(VerticalAlignment::Center),
            )),
        // 긴 구분선 — `Line`은 길이를 직접 준다(`Border` 1px는 폭을 부모에 맡긴다).
        Line::new()
            .x1(0.0)
            .y1(0.0)
            .x2(320.0)
            .y2(0.0)
            .stroke(Brush::from(ThemeBrush::CardStroke))
            .stroke_thickness(1.0),
    ))
}

/// 배지 3종 — 채운 알약 / 외곽선 알약 / 점 + 글자.
fn badges() -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((
            Border::new()
                .background(Brush::from(ThemeBrush::Accent))
                .corner_radius(CornerRadius::uniform(999.0))
                .padding(Thickness::new(10.0, 4.0, 10.0, 4.0))
                .content(
                    TextBlock::new()
                        .text("new")
                        .font_size(11.0)
                        .foreground(Color::rgb(255, 255, 255)),
                ),
            Border::new()
                .border_brush(Brush::from(ThemeBrush::CardStroke))
                .border_thickness(Thickness::uniform(1.0))
                .corner_radius(CornerRadius::uniform(999.0))
                .padding(Thickness::new(10.0, 4.0, 10.0, 4.0))
                .content(TextBlock::new().text("beta").font_size(11.0)),
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(6.0)
                .children((
                    Ellipse::new()
                        .width(8.0)
                        .height(8.0)
                        .fill(Brush::from(ThemeBrush::SystemCritical))
                        .vertical_alignment(VerticalAlignment::Center),
                    TextBlock::new().text("오류 3").font_size(11.0),
                )),
        ))
}

elm_magic::view! {
    fn Shapes() {
        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(radius_swatches());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(
                    StackPanel::new()
                        .spacing(12.0)
                        .children((per_corner_card(), accent_bar_card())),
                );
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(shapes());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(badges());
            }</Raw>
            <Text>"반지름 5단계 · 도형 4종 · 배지 3종"</Text>
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
        context.window_title("elm-magic — 모양과 테두리");
        View::component::<ElmView<Shapes>>(ElmInput::new(ShapesProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    fn plan_now() -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Shapes>(&mut ctx, &ShapesProps::default());
        plan(&tree)
    }

    /// 반지름 스케일은 오름차순이고, 마지막만 알약이다.
    #[test]
    fn the_radius_scale_ascends_and_ends_with_a_pill() {
        let radii: Vec<f64> = RADII.iter().map(|(radius, _)| *radius).collect();
        for pair in radii.windows(2) {
            assert!(pair[0] < pair[1], "{radii:?}는 오름차순이어야 한다");
        }
        assert!(!radii.iter().take(4).any(|radius| is_pill(*radius)));
        assert!(is_pill(radii[4]));
    }

    /// 알약 판정은 "높이보다 큰 값"이라는 관례를 코드로 옮긴 것이다.
    #[test]
    fn pill_detection_does_not_depend_on_exact_numbers() {
        assert!(is_pill(999.0));
        assert!(is_pill(100.0));
        assert!(!is_pill(99.9));
    }

    /// 구조: 스와치/표면/도형/배지 네 묶음은 Raw, 나머지는 elm 텍스트 하나뿐이다.
    #[test]
    fn the_plan_has_four_raw_blocks_and_no_buttons() {
        let (node, pass) = plan_now();
        assert_eq!(pass.count("Raw"), 4);
        assert_eq!(pass.count("Button"), 0, "이 예제는 상호작용이 없다");
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(kinds, vec!["Raw", "Raw", "Raw", "Raw", "TextBlock"]);
    }
}
