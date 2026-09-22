//! windows-reactor 예제 26 — **버튼 변형**: 스타일 4종 · 리소스 오버라이드 · 비활성 상태
//! **업스트림 대응**: `button-icon` · `app-bar-icon` · `radio-buttons` — 업스트림도
//! `Button::style` + `resource_overrides`(그리고 `AppBarButton`)로 변형을 만들고,
//! 비활성은 `is_enabled(false)`로 WinUI에 맡긴다.
//!
//!
//! **언제 쓰나**: "주요 동작 / 보조 동작 / 위험한 동작"을 구분해야 할 때.
//!
//! **Reactor가 주는 것은 두 가지뿐이다** (`generated.rs`의 `Button`)
//! - `Button::style(ButtonStyle::{Default, Accent, Subtle, TextLink})` — WinUI 기본 스타일.
//! - `Button::resource_overrides(ResourceOverrides)` — 그 컨트롤의 `Resources` 사전에 값을
//!   심어 **테마 리소스를 덮어쓴다**: `ButtonBackground` / `ButtonBorderBrush` /
//!   `ButtonForeground` / `ButtonCornerRadius`.
//!
//! 모양·크기·정렬은 `LayoutControl`(`is_enabled`, `horizontal_content_alignment`)로 조정한다.
//! **비활성 상태는 색을 만들지 않는다** — `is_enabled(false)`면 WinUI가 회색 표현까지
//! 알아서 한다(직접 회색을 칠하면 고대비 모드에서 깨진다).
//!
//! **변형을 코드로 정의한다** ([`VariantSpec`])
//! - 화면마다 `Button::new().style(..)`을 직접 쓰면 변형이 흩어진다 — 변형을 **표**로 두고
//!   화면은 이름만 고른다([`spec`]).
//! - `fill`/`line`이 `None`이면 **오버라이드를 심지 않는다**(테마 기본을 덮어쓰지 않는다).
//! - `ResourceOverrides`의 키는 **WinUI 리소스 이름**이다 — 오타가 나면 **조용히 무시**된다
//!   (컴파일 오류가 아니다). 그래서 키를 한 함수([`overrides`])에만 둔다.
//!
//! **주의**
//! - `<Raw>` 안의 버튼에는 **콜백을 붙이지 않는다** — Raw 클로저에는 `context`가 없어
//!   Reactor 콜백을 만들 수 없다(예제 08). 상호작용은 elm 쪽에 둔다.
//! - `style`/`resource_overrides`는 **`Button` 전용**이다. 다른 컨트롤은 `<Raw>`에서
//!   WinUI 속성으로 조정한다(예제 24).

use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, AppBarButton, AppBarButtonSlot, Button, ButtonStyle, ChildrenControl, Color, Component,
    ComponentContext, ContentControl, CornerRadius, DropDownButton, HorizontalAlignment,
    HyperlinkButton, KeyedView, Orientation, RepeatButton, ResourceOverrides, SlotsControl,
    SplitButton, StackPanel, Symbol, SymbolIcon, ToggleButton, TooltipExt, View, ViewContext,
};

/// 버튼 변형 4종 — 화면은 이 이름만 고른다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    /// 주요 동작 (한 화면에 하나).
    Primary,
    /// 보조 동작.
    Secondary,
    /// 되돌릴 수 없는 동작.
    Danger,
    /// 조용한 동작 (면 없이 글자만).
    Ghost,
}

impl Variant {
    /// 변형 순서 — 화면과 테스트가 같은 순서를 본다.
    const ALL: [Variant; 4] = [
        Variant::Primary,
        Variant::Secondary,
        Variant::Danger,
        Variant::Ghost,
    ];

    fn label(self) -> &'static str {
        match self {
            Variant::Primary => "Primary",
            Variant::Secondary => "Secondary",
            Variant::Danger => "위험",
            Variant::Ghost => "Ghost",
        }
    }
}

/// 변형의 시각 사양 — **순수 데이터**라 테스트가 그대로 검증한다.
#[derive(Clone, Copy, Debug, PartialEq)]
struct VariantSpec {
    /// WinUI 기본 스타일(먼저 쓴다).
    style: ButtonStyle,
    /// 채울 색 — `None`이면 오버라이드하지 않는다(테마 기본 유지).
    fill: Option<Color>,
    /// 글자색.
    text: Color,
    /// 테두리색 — `None`이면 테마 기본.
    line: Option<Color>,
    radius: f64,
}

/// 변형 → 사양. 브랜드 색은 여기서만 정의한다(예제 25의 팔레트와 같은 값).
fn spec(variant: Variant) -> VariantSpec {
    match variant {
        // 주요 동작: WinUI의 Accent 스타일을 그대로 쓴다 — 색을 새로 만들지 않는다.
        Variant::Primary => VariantSpec {
            style: ButtonStyle::Accent,
            fill: Some(Color::rgb(0, 95, 184)),
            text: Color::rgb(255, 255, 255),
            line: Some(Color::rgb(0, 95, 184)),
            radius: 4.0,
        },
        // 보조 동작: 면 없이 테두리만(브랜드 색).
        Variant::Secondary => VariantSpec {
            style: ButtonStyle::Default,
            fill: None,
            text: Color::rgb(0, 95, 184),
            line: Some(Color::rgb(0, 95, 184)),
            radius: 4.0,
        },
        // 위험한 동작: 시스템 오류 색 계열 — 고대비에서도 읽힌다.
        Variant::Danger => VariantSpec {
            style: ButtonStyle::Default,
            fill: Some(Color::rgb(196, 43, 28)),
            text: Color::rgb(255, 255, 255),
            line: Some(Color::rgb(196, 43, 28)),
            radius: 4.0,
        },
        // 조용한 동작: WinUI `Subtle` 스타일을 그대로(오버라이드 없음).
        Variant::Ghost => VariantSpec {
            style: ButtonStyle::Subtle,
            fill: None,
            text: Color::rgb(0, 95, 184),
            line: None,
            radius: 4.0,
        },
    }
}

/// 사양 → 오버라이드. **없는 값은 심지 않는다** — 키를 한 곳에만 두기 위한 함수다.
fn overrides(spec: VariantSpec) -> ResourceOverrides {
    let mut overrides =
        ResourceOverrides::new().set("ButtonCornerRadius", CornerRadius::uniform(spec.radius));
    if let Some(fill) = spec.fill {
        overrides = overrides.set("ButtonBackground", fill);
    }
    if let Some(line) = spec.line {
        overrides = overrides.set("ButtonBorderBrush", line);
    }
    overrides.set("ButtonForeground", spec.text)
}

/// 변형 버튼 하나 — 사양대로.
fn variant_button(variant: Variant, enabled: bool) -> View {
    let spec = spec(variant);
    Button::new()
        .style(spec.style)
        .resource_overrides(overrides(spec))
        .is_enabled(enabled)
        .horizontal_content_alignment(HorizontalAlignment::Center)
        .content(variant.label())
}

/// 변형 4종 — `disabled`면 전부 비활성(회색 표현은 WinUI가 만든다).
fn variant_row(disabled: bool) -> View {
    let cells: Vec<KeyedView> = Variant::ALL
        .iter()
        .enumerate()
        .map(|(index, variant)| KeyedView::new(index as u64, variant_button(*variant, !disabled)))
        .collect();

    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .keyed_children(cells)
}

/// 변형 말고도 WinUI에는 버튼이 여럿 있다 — 필요한 것을 골라 쓴다.
fn other_buttons() -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(12.0)
        .children((
            RepeatButton::new().content("RepeatButton"),
            DropDownButton::new().content("DropDownButton"),
            SplitButton::new().content("SplitButton"),
            HyperlinkButton::new()
                .content("HyperlinkButton")
                .tooltip("링크는 navigate_uri로 이동한다"),
            ToggleButton::new().is_checked(true).content("ToggleButton"),
            // 아이콘 버튼: 슬롯에 아이콘을 넣는다(글자 대신).
            AppBarButton::new().label("추가").slot(
                AppBarButtonSlot::Icon,
                SymbolIcon::new().symbol(Symbol::Add),
            ),
        ))
}

elm_magic::view! {
    fn Buttons(disabled = false) {
        // `<Raw>`는 그대로 복사된다 — 상태를 지역 변수로 옮겨 클로저가 캡처하게 한다(예제 21).
        let is_disabled = disabled;

        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(variant_row(is_disabled));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(other_buttons());
            }</Raw>

            <If when={is_disabled}>
                <Text>"상태: disabled"</Text>
                <Button on_click={disabled = false}>"활성화"</Button>
            <Else>
                <Text>"상태: enabled"</Text>
                <Button on_click={disabled = true}>"비활성화"</Button>
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
        context.window_title("elm-magic — 버튼 변형");
        View::component::<ElmView<Buttons>>(ElmInput::new(ButtonsProps::default()))
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

    fn plan_for(disabled: bool) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Buttons>(
            &mut ctx,
            &ButtonsProps {
                disabled: Some(disabled),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 주요/조용한 동작은 **WinUI 기본 스타일을 그대로** 쓴다(색을 새로 만들지 않는다).
    #[test]
    fn primary_and_ghost_reuse_winui_styles() {
        assert_eq!(spec(Variant::Primary).style, ButtonStyle::Accent);
        assert_eq!(spec(Variant::Ghost).style, ButtonStyle::Subtle);
    }

    /// 오버라이드를 심지 않는 변형은 테마 기본을 유지한다.
    #[test]
    fn ghost_does_not_paint_over_the_theme() {
        let ghost = spec(Variant::Ghost);
        assert!(ghost.fill.is_none(), "면을 칠하지 않는다");
        assert!(ghost.line.is_none(), "테두리도 칠하지 않는다");
    }

    /// 위험한 동작은 오류 색 계열이고, 글자는 대비가 확보된 흰색이다.
    #[test]
    fn danger_is_red_and_readable() {
        let danger = spec(Variant::Danger);
        assert_eq!(danger.fill, Some(Color::rgb(196, 43, 28)));
        assert_eq!(danger.fill, danger.line, "면과 선을 같은 색으로 둔다");
        assert_eq!(danger.text, Color::rgb(255, 255, 255));
    }

    /// 네 변형은 서로 다른 사양이다(표가 실제로 갈라져 있다).
    #[test]
    fn the_four_variants_are_distinct() {
        for (index, variant) in Variant::ALL.iter().enumerate() {
            for other in Variant::ALL.iter().skip(index + 1) {
                assert_ne!(
                    spec(*variant),
                    spec(*other),
                    "{variant:?}와 {other:?}의 사양이 같다"
                );
            }
        }
    }

    /// 반지름은 사양에서 온다 — 버튼마다 다르게 주지 않는다.
    #[test]
    fn every_variant_shares_one_radius() {
        let radii: Vec<f64> = Variant::ALL
            .iter()
            .map(|variant| spec(*variant).radius)
            .collect();
        assert!(
            radii.windows(2).all(|pair| pair[0] == pair[1]),
            "{radii:?}는 같은 값이어야 한다"
        );
    }

    /// 구조: 변형 행/다른 버튼 두 묶음은 Raw, 토글만 elm.
    #[test]
    fn the_plan_has_two_raw_blocks() {
        let (node, pass) = plan_for(false);
        assert_eq!(pass.count("Raw"), 2);
        assert_eq!(pass.count("Button"), 1);
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(kinds, vec!["Raw", "Raw", "TextBlock", "Button"]);
    }

    /// 비활성 토글이 화면 라벨에 반영된다.
    #[test]
    fn the_disabled_toggle_flips_the_label() {
        let mut app = elm_magic::mount!(Buttons);
        app.assert_text("상태: enabled");
        app.click("비활성화");
        app.assert_text("상태: disabled");
        app.click("활성화");
        app.assert_text("상태: enabled");
    }
}
