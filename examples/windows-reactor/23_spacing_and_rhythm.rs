//! windows-reactor 예제 23 — **간격과 리듬**: 4/8 DIP 그리드 · padding vs margin
//! **업스트림 대응**: `gallery` · `card` · `stacker` — 업스트림 샘플의 여백도
//! `StackPanel::spacing`/`Border::padding`/`LayoutControl::margin` **세 곳에서만** 나온다
//! (CSS 개념이 아니라 WinUI 배치 규칙이다).
//!
//!
//! **언제 쓰나**: 화면이 "왜 이렇게 답답하지/헐거우지" 할 때. 원인은 대개 **간격이
//! 화면마다 다르기 때문**이다 — 값을 고르기 전에 **단위(리듬)** 를 정한다.
//!
//! **WinUI의 간격은 세 곳에서 나온다** (섞어 쓰면 리듬이 깨진다)
//! 1. `StackPanel::spacing` — **형제 사이** 간격. 어댑터는 spacing을 넣지 않으므로
//!    (기본 0) `<Raw>`에서 정한다.
//! 2. `Border::padding` — **자기 안쪽** 여백(내용과 테두리 사이). 표면이 커진다.
//! 3. `LayoutControl::margin` — **자기 바깥** 여백. 표면 크기는 그대로, 자리만 밀린다.
//!
//! 셋의 의미는 CSS의 gap/padding/margin과 같다 — 그래서 `css!`를 안 쓰는 이 백엔드에서도
//! **개념은 그대로** 쓸 수 있다(값을 넣는 자리만 다르다).
//!
//! **`Thickness` 세 형태**: `uniform(v)` / `xy(가로, 세로)` / `new(left, top, right, bottom)`.
//! 1px 구분선은 `Border::new().height(1.0).background(테마 브러시)`로 만든다
//! (어댑터의 `<Divider>`가 하는 일과 같다).
//!
//! **규칙**
//! - 간격은 **단위의 배수**만 쓴다 — [`Rhythm`]이 그걸 강제하고 테스트가 고정한다.
//! - 밀도 전환은 **단위만** 바꾼다(8 ↔ 4). 화면 코드는 손대지 않는다.
//! - 목록의 간격은 `spacing`, 예외적인 여백만 `margin`으로 둔다(둘을 섞지 않는다).

use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Component, ComponentContext, ContentControl, CornerRadius,
    KeyedView, LayoutControl, Orientation, StackPanel, TextBlock, ThemeBrush, Thickness, View,
    ViewContext,
};

/// 간격 리듬 — 모든 여백이 `base`의 배수다.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rhythm {
    base: f64,
    xs: f64,
    sm: f64,
    md: f64,
    lg: f64,
    xl: f64,
}

impl Rhythm {
    /// 다섯 단계(작은 것 → 큰 것).
    fn steps(&self) -> [f64; 5] {
        [self.xs, self.sm, self.md, self.lg, self.xl]
    }

    /// 모든 단계가 단위의 배수인가 — 리듬의 정의를 코드로 옮긴 것.
    /// 화면은 값을 직접 쓰고, 이 술어는 `#[cfg(test)]`가 모든 밀도에서 단언한다
    /// — `allow`는 "테스트 전용 계약"이라는 표시다.
    #[allow(dead_code)]
    fn is_rhythmic(&self) -> bool {
        self.steps()
            .iter()
            .all(|step| (step / self.base).fract() == 0.0)
    }
}

/// 밀도 → 리듬. **단위만** 바뀐다(화면 코드는 그대로다).
fn rhythm(dense: bool) -> Rhythm {
    let base = if dense { 4.0 } else { 8.0 };
    Rhythm {
        base,
        xs: base,
        sm: base * 2.0,
        md: base * 3.0,
        lg: base * 4.0,
        xl: base * 6.0,
    }
}

/// 간격 스케일을 눈으로 — 각 단계만큼 긴 막대 + 실제 값.
fn scale_bars(rhythm: Rhythm) -> View {
    let rows: Vec<KeyedView> = rhythm
        .steps()
        .iter()
        .enumerate()
        .map(|(index, step)| {
            KeyedView::new(
                index as u64,
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(rhythm.sm)
                    .children((
                        Border::new()
                            .width(*step)
                            .height(12.0)
                            .corner_radius(CornerRadius::uniform(2.0))
                            .background(Brush::from(ThemeBrush::Accent)),
                        TextBlock::new()
                            .text(format!(
                                "{step} DIP = base ×{}",
                                (step / rhythm.base) as i32
                            ))
                            .font_size(12.0),
                    )),
            )
        })
        .collect();
    StackPanel::new().spacing(rhythm.xs).keyed_children(rows)
}

/// padding(안쪽) vs margin(바깥) — 같은 값, 다른 결과.
fn padding_vs_margin(rhythm: Rhythm) -> View {
    let card = |margin: bool| {
        let border = Border::new()
            .background(Brush::from(ThemeBrush::CardBackground))
            .border_brush(Brush::from(ThemeBrush::CardStroke))
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(CornerRadius::uniform(4.0));
        let border = if margin {
            border.margin(Thickness::uniform(rhythm.lg))
        } else {
            border.padding(Thickness::uniform(rhythm.lg))
        };
        border.content(TextBlock::new().text("내용").font_size(12.0))
    };

    StackPanel::new().spacing(rhythm.md).children((
        StackPanel::new().spacing(rhythm.xs).children((
            TextBlock::new()
                .text("padding — 표면이 그만큼 커진다")
                .font_size(11.0)
                .opacity(0.6),
            card(false),
        )),
        StackPanel::new().spacing(rhythm.xs).children((
            TextBlock::new()
                .text("margin — 표면 크기는 그대로, 자리만 밀린다")
                .font_size(11.0)
                .opacity(0.6),
            card(true),
        )),
    ))
}

/// `Thickness` 세 형태 + 1px 구분선.
fn thickness_forms(rhythm: Rhythm) -> View {
    let padded = |label: &str, thickness: Thickness| {
        StackPanel::new().spacing(rhythm.xs / 2.0).children((
            TextBlock::new().text(label).font_size(11.0).opacity(0.6),
            Border::new()
                .background(Brush::from(ThemeBrush::CardBackground))
                .border_brush(Brush::from(ThemeBrush::CardStroke))
                .border_thickness(Thickness::uniform(1.0))
                .padding(thickness)
                .content(TextBlock::new().text("내용").font_size(12.0)),
        ))
    };

    StackPanel::new().spacing(rhythm.sm).children((
        padded(
            "uniform(base) — 사방 같은 값",
            Thickness::uniform(rhythm.base),
        ),
        padded(
            "xy(lg, xs) — 가로 / 세로",
            Thickness::xy(rhythm.lg, rhythm.xs),
        ),
        padded(
            "new(lg, sm, 0, sm) — 변마다 (오른쪽 0)",
            Thickness::new(rhythm.lg, rhythm.sm, 0.0, rhythm.sm),
        ),
        // 1px 구분선 — 어댑터의 `<Divider>`가 만드는 것과 같은 모양이다.
        Border::new()
            .height(1.0)
            .background(Brush::from(ThemeBrush::CardStroke)),
    ))
}

elm_magic::view! {
    fn Spacing(dense = false) {
        // `<Raw>`가 캡처할 값은 렌더 본문에서 계산한다(예제 21).
        let rhythm = rhythm(dense);

        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(scale_bars(rhythm));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(padding_vs_margin(rhythm));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(thickness_forms(rhythm));
            }</Raw>

            <If when={dense}>
                <Text>"단위: 4 DIP"</Text>
                <Button on_click={dense = false}>"단위 8"</Button>
            <Else>
                <Text>"단위: 8 DIP"</Text>
                <Button on_click={dense = true}>"단위 4"</Button>
            </Else>
            </If>
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
        context.window_title("elm-magic — 간격 리듬");
        View::component::<ElmView<Spacing>>(ElmInput::new(SpacingProps::default()))
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

    fn plan_for(dense: bool) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Spacing>(
            &mut ctx,
            &SpacingProps {
                dense: Some(dense),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 리듬의 정의: 모든 단계가 단위의 배수다.
    #[test]
    fn every_step_is_a_multiple_of_the_base_unit() {
        for dense in [false, true] {
            let r = rhythm(dense);
            assert!(r.is_rhythmic(), "base={}에서 리듬이 깨졌다", r.base);
            assert_eq!(r.base, if dense { 4.0 } else { 8.0 });
        }
    }

    /// 단계는 오름차순이고, 밀도가 높으면 **모든 단계**가 작아진다.
    #[test]
    fn steps_ascend_and_density_scales_them_all() {
        let roomy = rhythm(false);
        let dense = rhythm(true);

        let steps = roomy.steps();
        for pair in steps.windows(2) {
            assert!(pair[0] < pair[1], "{:?}는 오름차순이어야 한다", steps);
        }

        for (roomy_step, dense_step) in roomy.steps().iter().zip(dense.steps().iter()) {
            assert!(dense_step < roomy_step, "4 DIP 그리드가 더 조밀해야 한다");
        }
    }

    /// 구조: 스케일/여백 비교/Thickness 세 묶음은 Raw, 토글만 elm.
    #[test]
    fn the_plan_has_three_raw_blocks() {
        let (node, pass) = plan_for(false);
        assert_eq!(pass.count("Raw"), 3);
        assert_eq!(pass.count("Button"), 1);
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(kinds, vec!["Raw", "Raw", "Raw", "TextBlock", "Button"]);
    }

    /// 단위 전환이 화면 라벨에 반영된다.
    #[test]
    fn the_unit_toggle_flips_the_label() {
        let mut app = elm_magic::mount!(Spacing);
        app.assert_text("단위: 8 DIP");
        app.click("단위 4");
        app.assert_text("단위: 4 DIP");
        app.click("단위 8");
        app.assert_text("단위: 8 DIP");
    }
}
