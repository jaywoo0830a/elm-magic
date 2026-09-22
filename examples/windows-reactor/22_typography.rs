//! windows-reactor 예제 22 — **타이포그래피**: 크기 스케일 · 굵기 · 줄바꿈/말줄임
//!
//! **언제 쓰나**: 글자 크기가 화면마다 제각각이 되기 시작할 때.
//!
//! **Reactor 0.100이 노출하는 글자 API는 이게 전부다**
//! - `TextBlock::{font_size, font_weight, foreground, text_wrapping, max_lines,
//!   text_trimming, is_text_selection_enabled}`.
//! - **폰트 패밀리/행간(line-height)은 없다** — WinUI 수준 조정이 필요하면 `<Raw>`에서
//!   `FontFamily`를 직접 다루거나(Reactor가 감싸지 않음) 기본값을 받아들인다.
//!   그래서 "타이포 시스템"은 **크기 · 굵기 · 색 · 자름** 네 축으로 만든다.
//!
//! **스케일을 먼저 정한다**
//! - 크기는 화면마다 고르지 않는다 — [`TypeScale`] 다섯 단계(display/title/subtitle/
//!   body/caption)만 쓴다. 밀도(comfortable/compact)는 **스케일 자체를 바꾼다**.
//! - 굵기는 `FontWeight` 상수 9단계가 전부다([`WEIGHTS`]) — 임의 숫자를 쓰지 않는다.
//! - 보조 텍스트는 **색을 하나 더 만들지 않고 `opacity`**로 만든다(`LayoutControl`).
//!
//! **줄바꿈/말줄임은 항상 짝으로**
//! - `text_wrapping(NoWrap)` + `text_trimming(WordEllipsis)` = 한 줄 말줄임.
//! - `text_wrapping(Wrap)` + `max_lines(2)` = 두 줄까지만.
//! - 이 둘을 안 정하면 WinUI 기본값(줄바꿈 없음, 자르지 않음)이라 **긴 글이 컨트롤을
//!   넘치거나 잘린다** — 잘림은 조용히 일어난다.
//!
//! **`<Raw>`와 스케일**: `<Raw>` 본문은 그대로 복사되므로 그 안에서 elm 상태를 읽을 수
//! 없다(예제 21). **렌더 본문에서 `let scale = type_scale(compact);`로 계산**해 캡처한다.
//! 목록처럼 길이가 변하는 내용은 `keyed_children` + `KeyedView`(키는 인덱스)로 만든다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, ChildrenControl, Component, ComponentContext, FontWeight, KeyedView, LayoutControl,
    StackPanel, TextBlock, TextTrimming, TextWrapping, View, ViewContext,
};

/// 타이포 스케일 — 화면 전체가 쓰는 다섯 크기(DIP).
#[derive(Clone, Copy, Debug, PartialEq)]
struct TypeScale {
    display: f64,
    title: f64,
    subtitle: f64,
    body: f64,
    caption: f64,
}

/// 밀도 → 스케일. **순수 함수**라 헤드리스 테스트가 그대로 검증한다.
///
/// compact는 "같은 비율로 조금씩 작게"가 아니라 **의도적으로 다른 스케일**이다
/// (정보 밀도가 높은 화면은 계층 차이도 줄여야 한다).
fn type_scale(compact: bool) -> TypeScale {
    if compact {
        TypeScale {
            display: 22.0,
            title: 17.0,
            subtitle: 14.0,
            body: 12.0,
            caption: 11.0,
        }
    } else {
        TypeScale {
            display: 28.0,
            title: 20.0,
            subtitle: 16.0,
            body: 14.0,
            caption: 12.0,
        }
    }
}

/// `FontWeight`는 OpenType 1..999 전 구간을 상수로 노출한다 — 이 9개면 충분하다.
const WEIGHTS: [(FontWeight, &str); 9] = [
    (FontWeight::THIN, "Thin 100"),
    (FontWeight::EXTRA_LIGHT, "ExtraLight 200"),
    (FontWeight::LIGHT, "Light 300"),
    (FontWeight::SEMI_LIGHT, "SemiLight 350"),
    (FontWeight::NORMAL, "Normal 400"),
    (FontWeight::MEDIUM, "Medium 500"),
    (FontWeight::SEMI_BOLD, "SemiBold 600"),
    (FontWeight::BOLD, "Bold 700"),
    (FontWeight::BLACK, "Black 900"),
];

/// 잘림/줄바꿈을 보여줄 긴 문장.
const LONG: &str = "이 문장은 컨트롤 폭보다 길어서 줄바꿈이나 말줄임 규칙이 없으면 \
                    조용히 잘리거나 넘친다 — 그래서 타이포 시스템은 자름 규칙까지 정한다.";

/// 크기 한 단계 = 샘플 + 실제 DIP 값. 라벨을 손으로 적지 않고 스케일에서 만든다.
fn sample(size: f64, text: &str) -> View {
    StackPanel::new().spacing(2.0).children((
        TextBlock::new()
            .text(text)
            .font_size(size)
            .font_weight(FontWeight::SEMI_BOLD),
        TextBlock::new()
            .text(format!("{size} DIP"))
            .font_size(11.0)
            .opacity(0.6),
    ))
}

/// 다섯 단계를 위에서 아래로.
fn scale_column(scale: TypeScale) -> View {
    StackPanel::new().spacing(12.0).children((
        sample(scale.display, "Aa 디자인 토큰"),
        sample(scale.title, "Aa 섹션 제목"),
        sample(scale.subtitle, "Aa 카드 제목"),
        sample(scale.body, "Aa 본문 — 기본 크기"),
        sample(scale.caption, "Aa 보조 설명"),
    ))
}

/// 굵기 9단계 — 길이가 고정이지만 `keyed_children`으로 만들어 본다(동적 목록의 기본형).
fn weight_column() -> View {
    let rows: Vec<KeyedView> = WEIGHTS
        .iter()
        .enumerate()
        .map(|(index, (weight, name))| {
            KeyedView::new(
                index as u64,
                TextBlock::new()
                    .text(format!("{name} — 가나다 Aa"))
                    .font_size(14.0)
                    .font_weight(*weight),
            )
        })
        .collect();
    StackPanel::new().spacing(4.0).keyed_children(rows)
}

/// 줄바꿈/말줄임 — 같은 문장, 다른 규칙. 규칙이 없으면 WinUI 기본값으로 잘린다.
fn text_handling() -> View {
    StackPanel::new().spacing(10.0).children((
        TextBlock::new()
            .text("한 줄 말줄임 — NoWrap + WordEllipsis")
            .font_size(11.0)
            .opacity(0.6),
        TextBlock::new()
            .text(LONG)
            .font_size(14.0)
            .width(420.0)
            .text_wrapping(TextWrapping::NoWrap)
            .text_trimming(TextTrimming::WordEllipsis),
        TextBlock::new()
            .text("두 줄까지만 — Wrap + max_lines 2")
            .font_size(11.0)
            .opacity(0.6),
        TextBlock::new()
            .text(LONG)
            .font_size(14.0)
            .width(420.0)
            .text_wrapping(TextWrapping::Wrap)
            .max_lines(2),
    ))
}

elm_magic::view! {
    fn Typography(compact = false) {
        // `<Raw>`는 그대로 복사된다 — 스케일을 여기서 계산해 클로저가 캡처한다(예제 21).
        let scale = type_scale(compact);

        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(scale_column(scale));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(weight_column());
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(text_handling());
            }</Raw>

            <If when={compact}>
                <Text>"밀도: compact"</Text>
                <Button on_click={compact = false}>"여유롭게"</Button>
            <Else>
                <Text>"밀도: comfortable"</Text>
                <Button on_click={compact = true}>"조밀하게"</Button>
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
        context.window_title("elm-magic — 타이포그래피");
        // props를 넘기지 않는다(`Default`) — 밀도는 **elm이 소유**한다(예제 18과 같은 방식).
        View::component::<ElmView<Typography>>(ElmInput::new(TypographyProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    fn plan_for(compact: bool) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Typography>(
            &mut ctx,
            &TypographyProps {
                compact: Some(compact),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 스케일은 내림차순이어야 계층이 읽힌다.
    #[test]
    fn the_scale_is_strictly_descending() {
        for compact in [false, true] {
            let s = type_scale(compact);
            assert!(
                s.display > s.title
                    && s.title > s.subtitle
                    && s.subtitle > s.body
                    && s.body > s.caption,
                "밀도 compact={compact}에서 계층이 무너졌다"
            );
        }
    }

    /// 밀도는 **모든 단계**를 줄인다 — 한 단계만 줄이면 계층 차이가 사라진다.
    #[test]
    fn compact_shrinks_every_step() {
        let roomy = type_scale(false);
        let dense = type_scale(true);
        assert!(dense.display < roomy.display);
        assert!(dense.title < roomy.title);
        assert!(dense.subtitle < roomy.subtitle);
        assert!(dense.body < roomy.body);
        assert!(dense.caption < roomy.caption);
    }

    /// 굵기 9단계는 서로 다르고 오름차순이다(OpenType 값 그대로).
    #[test]
    fn weights_are_distinct_and_ordered() {
        assert_eq!(WEIGHTS.len(), 9);
        for pair in WEIGHTS.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{} 다음이 {}", pair[0].1, pair[1].1);
        }
    }

    /// 구조: 스케일/굵기/자름 세 묶음은 Raw가 만들고, 밀도 토글만 elm이 그린다.
    #[test]
    fn the_plan_has_three_raw_blocks() {
        let (node, pass) = plan_for(false);
        assert_eq!(pass.count("Raw"), 3);
        assert_eq!(pass.count("Button"), 1);
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(kinds, vec!["Raw", "Raw", "Raw", "TextBlock", "Button"]);
    }

    /// 밀도 토글이 실제로 화면을 바꾼다(헤드리스에서 클릭까지 확인).
    #[test]
    fn the_density_toggle_flips_the_label() {
        let mut app = elm_magic::mount!(Typography);
        app.assert_text("밀도: comfortable");
        app.click("조밀하게");
        app.assert_text("밀도: compact");
        app.click("여유롭게");
        app.assert_text("밀도: comfortable");
    }
}
