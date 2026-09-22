//! windows-reactor 예제 25 — **색 체계**: 테마 브러시 8종 · 브랜드 팔레트 · 대비 규칙
//!
//! **언제 쓰나**: 색을 화면마다 고르기 시작했을 때. 색은 **고르는 것이 아니라 정하는 것**이다.
//!
//! **Reactor가 노출하는 테마 브러시는 8개가 전부다** ([`THEME_BRUSHES`])
//!
//! | 브러시 | 쓰는 곳 |
//! |---|---|
//! | `Accent` / `AccentText` | 주요 동작 / 액센트 **위의** 글자 |
//! | `PrimaryText` | 본문 글자 |
//! | `SolidBackground` | 창·헤더 배경 |
//! | `CardBackground` / `CardStroke` | 카드 표면 / 테두리·구분선 |
//! | `SystemCritical` / `SystemCriticalBackground` | 오류 강조 / 오류 배경 |
//!
//! 이 8개는 **이름일 뿐 값이 아니다** — 라이트/다크/고대비를 WinUI가 알아서 바꾼다.
//! 그래서 "테마 지원"은 **테마 브러시를 쓰는 것**으로 자동으로 따라온다(예제 21).
//!
//! **그 밖의 색은 두 가지 방법으로만 만든다**
//! 1. **브랜드 팔레트를 정해서** 쓴다([`BRAND`]) — 톤(50/100/500/700/900)을 미리 정하고
//!    화면은 톤 이름만 고른다. Reactor에 색 계산 API는 없으므로 값은 손으로 정한다.
//! 2. **투명도**(`Color::argb`)로 같은 색을 단계로 나눈다 — 색을 더 만들지 않는다.
//!
//! **글자색은 눈으로 고르지 않는다** — [`readable_on`]이 배경 휘도로 흰/검을 정한다
//! (WCAG 상대 휘도). 헤드리스 테스트가 이 규칙을 고정한다.
//!
//! **주의**: `Color`는 **알파를 포함한 8비트 ARGB**다. `Color::rgb`는 불투명(255),
//! `Color::argb(a, ..)`는 알파를 직접 준다. `Brush::from(ThemeBrush)`와
//! `Brush::from(Color)`가 둘 다 `Brush`로 들어가므로 함수 인자 하나로 받을 수 있다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Color, Component, ComponentContext, ContentControl,
    CornerRadius, FontWeight, KeyedView, LayoutControl, Orientation, StackPanel, TextBlock,
    ThemeBrush, Thickness, VariableSizedWrapGrid, View, ViewContext,
};

/// 테마 브러시 8종 — 이름과 **쓰는 곳**을 함께 둔다(팀이 색을 고르지 않게).
const THEME_BRUSHES: [(ThemeBrush, &str, &str); 8] = [
    (ThemeBrush::Accent, "Accent", "주요 동작 · 선택"),
    (ThemeBrush::AccentText, "AccentText", "액센트 위의 글자"),
    (ThemeBrush::PrimaryText, "PrimaryText", "본문 글자"),
    (
        ThemeBrush::SolidBackground,
        "SolidBackground",
        "창 · 헤더 배경",
    ),
    (ThemeBrush::CardBackground, "CardBackground", "카드 표면"),
    (ThemeBrush::CardStroke, "CardStroke", "카드 테두리 · 구분선"),
    (ThemeBrush::SystemCritical, "SystemCritical", "오류 강조"),
    (
        ThemeBrush::SystemCriticalBackground,
        "SystemCriticalBackground",
        "오류 배경",
    ),
];

/// 브랜드 팔레트 — 톤 이름으로만 쓰고, 화면에는 값을 적지 않는다.
const BRAND: [Color; 5] = [
    Color::rgb(235, 244, 255), // 50
    Color::rgb(168, 208, 255), // 100
    Color::rgb(0, 95, 184),    // 500
    Color::rgb(0, 62, 122),    // 700
    Color::rgb(0, 32, 64),     // 900
];

/// 팔레트 톤 이름.
const BRAND_TONES: [&str; 5] = ["50", "100", "500", "700", "900"];

/// WCAG 상대 휘도 (0 = 검정, 1 = 흰색).
///
/// sRGB 채널을 선형화한 뒤 가중 평균한다 — "이 색 위에 흰 글자가 읽히나"를
/// 사람 눈이 아니라 **계산으로** 정하기 위한 값이다.
fn luminance(color: Color) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

/// 배경 위에서 읽히는 글자색 — 흰색/검정 중 하나를 **계산으로** 고른다.
fn readable_on(background: Color) -> Color {
    if luminance(background) > 0.5 {
        Color::rgb(0, 0, 0)
    } else {
        Color::rgb(255, 255, 255)
    }
}

/// 테마 브러시 카탈로그 — 색 면 + 이름 + 쓰는 곳.
fn theme_swatches() -> View {
    let cells: Vec<KeyedView> = THEME_BRUSHES
        .iter()
        .enumerate()
        .map(|(index, (brush, name, use_case))| {
            KeyedView::new(
                index as u64,
                StackPanel::new().spacing(4.0).children((
                    Border::new()
                        .width(120.0)
                        .height(36.0)
                        .background(Brush::from(*brush))
                        .border_brush(Brush::from(ThemeBrush::CardStroke))
                        .border_thickness(Thickness::uniform(1.0))
                        .corner_radius(CornerRadius::uniform(4.0)),
                    TextBlock::new()
                        .text(*name)
                        .font_size(11.0)
                        .font_weight(FontWeight::SEMI_BOLD),
                    TextBlock::new()
                        .text(*use_case)
                        .font_size(10.0)
                        .opacity(0.6),
                )),
            )
        })
        .collect();

    VariableSizedWrapGrid::new()
        .item_width(150.0)
        .item_height(80.0)
        .orientation(Orientation::Horizontal)
        .keyed_children(cells)
}

/// 브랜드 팔레트 + 대비 규칙 — 글자색은 **계산해서** 넣는다.
fn brand_palette() -> View {
    let cells: Vec<KeyedView> = BRAND
        .iter()
        .zip(BRAND_TONES)
        .enumerate()
        .map(|(index, (color, tone))| {
            KeyedView::new(
                index as u64,
                StackPanel::new().spacing(4.0).children((
                    Border::new()
                        .width(120.0)
                        .height(36.0)
                        .background(*color)
                        .corner_radius(CornerRadius::uniform(4.0))
                        .padding(Thickness::uniform(6.0))
                        .content(
                            TextBlock::new()
                                .text("Aa")
                                .font_size(12.0)
                                .foreground(readable_on(*color)),
                        ),
                    TextBlock::new()
                        .text(format!("brand {tone}"))
                        .font_size(10.0)
                        .opacity(0.6),
                )),
            )
        })
        .collect();

    VariableSizedWrapGrid::new()
        .item_width(150.0)
        .item_height(72.0)
        .orientation(Orientation::Horizontal)
        .keyed_children(cells)
}

/// 투명도 — 색을 더 만들지 않고 같은 색을 단계로 나눈다.
fn alpha_steps() -> View {
    let cells: Vec<KeyedView> = [255u8, 192, 128, 64]
        .iter()
        .enumerate()
        .map(|(index, alpha)| {
            KeyedView::new(
                index as u64,
                StackPanel::new().spacing(4.0).children((
                    Border::new()
                        .width(72.0)
                        .height(28.0)
                        .background(Color::argb(*alpha, 0, 95, 184))
                        .corner_radius(CornerRadius::uniform(4.0)),
                    TextBlock::new()
                        .text(format!("alpha {alpha}"))
                        .font_size(10.0)
                        .opacity(0.6),
                )),
            )
        })
        .collect();

    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .keyed_children(cells)
}

/// 시스템 색의 쓰임 — 오류 표면. 고대비 모드에서도 대비가 보장되는 조합이다.
fn severity_surface() -> View {
    Border::new()
        .background(Brush::from(ThemeBrush::SystemCriticalBackground))
        .border_brush(Brush::from(ThemeBrush::SystemCritical))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(4.0))
        .padding(Thickness::uniform(12.0))
        .content(
            StackPanel::new().spacing(2.0).children((
                TextBlock::new()
                    .text("SystemCriticalBackground + SystemCritical")
                    .font_size(11.0)
                    .font_weight(FontWeight::SEMI_BOLD),
                TextBlock::new()
                    .text("오류 표면은 새 색을 만들지 않고 시스템 색을 쓴다")
                    .font_size(11.0)
                    .opacity(0.7),
            )),
        )
}

elm_magic::view! {
    fn Palette() {
        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(theme_swatches());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(brand_palette());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(alpha_steps());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(severity_surface());
            }</Raw>
            <Text>"테마 브러시 8종 · 브랜드 5톤 · 알파 4단계"</Text>
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
        context.window_title("elm-magic — 색 체계");
        View::component::<ElmView<Palette>>(ElmInput::new(PaletteProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    fn plan_now() -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Palette>(&mut ctx, &PaletteProps::default());
        plan(&tree)
    }

    /// 상대 휘도의 양 끝은 흰색(1)과 검정(0)이다.
    #[test]
    fn luminance_spans_black_to_white() {
        assert!((luminance(Color::rgb(255, 255, 255)) - 1.0).abs() < 1e-6);
        assert!(luminance(Color::rgb(0, 0, 0)).abs() < 1e-6);
        // 중간 회색은 0.5가 아니라 0.22 근처다 — sRGB 선형화의 결과다.
        assert!(luminance(Color::rgb(128, 128, 128)) < 0.3);
    }

    /// 대비 규칙: 밝은 배경에는 검은 글자, 어두운 배경에는 흰 글자.
    #[test]
    fn readable_on_picks_black_or_white_by_luminance() {
        assert_eq!(readable_on(Color::rgb(255, 255, 255)), Color::rgb(0, 0, 0));
        assert_eq!(readable_on(Color::rgb(0, 0, 0)), Color::rgb(255, 255, 255));
        assert_eq!(
            readable_on(BRAND[0]),
            Color::rgb(0, 0, 0),
            "톤 50은 아주 밝다"
        );
        assert_eq!(
            readable_on(BRAND[4]),
            Color::rgb(255, 255, 255),
            "톤 900은 아주 어둡다"
        );
    }

    /// 팔레트는 톤이 올라갈수록 어두워진다(휘도 단조 감소).
    #[test]
    fn the_brand_palette_darkens_as_the_tone_rises() {
        let tones: Vec<f64> = BRAND.iter().map(|color| luminance(*color)).collect();
        for pair in tones.windows(2) {
            assert!(pair[0] > pair[1], "{tones:?}는 어두워져야 한다");
        }
    }

    /// 카탈로그는 8종이 전부이고, 팔레트와 톤 이름 개수가 맞는다.
    #[test]
    fn the_catalog_is_complete() {
        assert_eq!(THEME_BRUSHES.len(), 8, "테마 브러시는 8개가 전부다");
        assert_eq!(BRAND.len(), BRAND_TONES.len());
    }

    /// 구조: 카탈로그/팔레트/알파/오류 표면 네 묶음은 Raw가 만든다.
    #[test]
    fn the_plan_has_four_raw_blocks() {
        let (node, pass) = plan_now();
        assert_eq!(pass.count("Raw"), 4);
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(kinds, vec!["Raw", "Raw", "Raw", "Raw", "TextBlock"]);
    }
}
