//! windows-reactor 예제 16 — **`css!` 없이 스타일하기** (WinUI 테마/리소스)
//!
//! **이 어댑터는 스타일 계층을 지원하지 않는다** — 의도다.
//! `css!`/`class`는 어댑터가 **읽지 않으므로** 이 백엔드에서 아무 효과가 없다:
//!
//! ```ignore
//! elm_magic::css! { .card { bg: surface; padding: 16; } }   // ← 무시된다
//! <Col class="card"> … </Col>                                // ← class도 무시된다
//! ```
//!
//! **대신 무엇을 쓰나** (검증된 WinUI 속성만)
//! - **테마 브러시**: `ThemeBrush::{Accent, AccentText, PrimaryText, SolidBackground,
//!   CardBackground, CardStroke, SystemCritical, SystemCriticalBackground}` —
//!   WinUI 테마 리소스를 그대로 참조하므로 라이트/다크/고대비가 **자동으로** 따라온다.
//! - **직접 색**: `Color::rgb(r, g, b)` / `Color::argb(a, r, g, b)` → `Brush`.
//! - **간격/모양**: `StackPanel::spacing`, `Border::{padding, corner_radius,
//!   border_thickness, background, border_brush}`, `LayoutControl::{width, height,
//!   min_width, max_width}`.
//! - **글자**: `TextBlock::{font_size, font_weight, foreground}` —
//!   `FontWeight::{BOLD, SEMI_BOLD, NORMAL, LIGHT, …}`.
//! - **컨트롤 스타일**: `Button::style(ButtonStyle::…)`, `ResourceOverrides`.
//!
//! **어디에 쓰나**: 전부 `<Raw>` 안이다(예제 08). 어댑터는 계획을 컨트롤로 옮기기만
//! 하므로, WinUI 수준 조정은 Raw가 유일한 통로다.
//!
//! **베스트 패턴**
//! - 색을 **하드코딩하지 않는다** — `ThemeBrush`가 곧 "테마 = 팔레트"다.
//! - 반복되는 스타일은 **함수로 뽑는다**(`fn card(content: impl Into<View>) -> View`).
//!   egui/gpui의 `css!` + 클래스 조합에 대응하는, WinUI에서 자연스러운 재사용 단위다.
//! - 같은 조합이 여러 화면에 필요하면 **어댑터의 계획 층에 매핑을 추가**하는 편이 낫다
//!   (`crates/elm-magic-windows-reactor/src/plan.rs`).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Component, ComponentContext, ContentControl, CornerRadius,
    FontWeight, StackPanel, TextBlock, ThemeBrush, Thickness, View, ViewContext,
};

/// 반복되는 "카드" 스타일 — WinUI에서는 **함수**가 재사용 단위다.
fn card(content: impl Into<View>) -> View {
    Border::new()
        .background(Brush::from(ThemeBrush::CardBackground))
        .border_brush(Brush::from(ThemeBrush::CardStroke))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(8.0))
        .padding(Thickness::uniform(12.0))
        .content(content)
}

elm_magic::view! {
    fn Styled(count = 0) {
        <Col>
            // 어댑터는 여기서 아무 스타일도 적용하지 않는다 — 전부 Raw가 만든다.
            <Raw>|out: &mut RawSlot| {
                let body = StackPanel::new()
                    .spacing(8.0)
                    .children((
                        TextBlock::new()
                            .text("CardBackground + CardStroke")
                            .font_weight(FontWeight::SEMI_BOLD),
                        TextBlock::new()
                            .text("WinUI 테마를 그대로 따른다")
                            .font_size(12.0),
                    ));
                *out = Some(card(body));
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
        context.window_title("elm-magic — WinUI 스타일");
        View::component::<ElmView<Styled>>(ElmInput::new(StyledProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    /// 어댑터가 클래스를 **읽지 않는다**는 사실을 계약으로 고정한다.
    #[test]
    fn classes_do_not_change_the_plan() {
        let plain = elm_magic::ui! { <Col class="card">"x"</Col> };
        let styled = elm_magic::ui! { <Col class="card shadow-lg">"x"</Col> };

        let (a, _) = plan(&plain);
        let (b, _) = plan(&styled);
        assert_eq!(a.control(), b.control());
        assert_eq!(a.children.len(), b.children.len());
        // 스타일이 계획에 실리지 않는다 → css!를 바꿔도 WinUI 결과는 같다.
        assert_eq!(a.text, b.text);
    }

    #[test]
    fn semantic_widgets_still_carry_meaning() {
        // `<Strong>`은 굵게, `<Banner kind>`는 severity로 — 의미는 스타일 계층 없이도 전달된다.
        let (node, pass) = plan(&elm_magic::ui! {
            <Col>
                <Strong>"제목"</Strong>
                <Banner kind="warn">"주의"</Banner>
            </Col>
        });
        assert_eq!(pass.count("InfoBar"), 1);
        assert!(matches!(
            node.children[0].kind,
            elm_magic_windows_reactor::PlanKind::Text { strong: true }
        ));
    }
}
