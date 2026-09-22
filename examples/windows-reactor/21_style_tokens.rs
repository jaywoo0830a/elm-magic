//! windows-reactor 예제 21 — **디자인 토큰**과 테마 (라이트/다크를 한 곳에서)
//!
//! 예제 16은 "무엇이 **안** 되는가"를 다룬다 — 이 백엔드는 `css!`/`class`를 읽지 않는다.
//! 이 예제는 그다음 질문에 답한다: **그럼 WinUI에서 스타일을 어떻게 관리하는가.**
//!
//! **두 층으로 나눈다**
//! 1. **토큰**([`Tokens`]) — 반지름/간격/글자 크기/색. 순수 함수 [`tokens`]가 테마
//!    하나로 정한다. 화면 코드에는 숫자도 색도 없다(전부 토큰 이름이다).
//! 2. **스타일 함수**([`header`]/[`card_row`]/[`branded_button`]) — 토큰을 받아
//!    `View`를 만든다. egui/gpui의 `css!` + 클래스 조합에 대응하는 WinUI의 재사용
//!    단위다(예제 16과 같은 결론: 반복은 **함수**로 뽑는다).
//!
//! **색을 고르는 두 가지 규칙**
//! - **테마 브러시가 우선**: `ThemeBrush::{CardBackground, CardStroke, SolidBackground,
//!   PrimaryText, Accent}` — 라이트/다크/고대비를 WinUI가 알아서 따라온다(`surface`).
//! - **브랜드 색은 테마마다 직접**: Reactor가 노출하는 테마 브러시는 8개뿐이라 그 밖의
//!   색은 `Color::rgb`로 준다. 이 값은 테마를 따라가지 **않으므로** 토큰에 넣어
//!   테마별로 한 곳에서 관리한다(`brand`/`on_brand`).
//!
//! **테마는 호스트가 소유한다** (이 예제의 핵심 계약)
//! - 창의 라이트/다크는 **WinUI 창 속성**이다 — 호스트가 `ViewContext::window_visuals`로
//!   정하고(`WindowTheme::{System, Light, Dark}`), 제목 표시줄까지 함께 바뀐다.
//!   elm은 창을 모른다(어댑터가 옮기지 않는다) — 그래서 **elm이 스스로 테마를 바꿀 수 없다**.
//! - 그래서 상태는 호스트 컴포넌트(`Root`)가 소유하고, elm에게는 props(`dark`)로
//!   내려보내며, elm은 "어느 테마로 바꿀지"만 콜백 prop(`on_toggle: fn(bool)`)으로
//!   알린다(예제 11과 같은 계약).
//!
//! **`<Raw>`와 토큰** (여기서 자주 걸린다)
//! - 스타일은 `<Raw>` 안에서만 만들 수 있다(예제 08/16). 그런데 `<Raw>` 본문은 매크로가
//!   **그대로 복사**하므로 그 안에서 elm 상태(`dark`)를 읽을 수 없다.
//! - 그래서 **렌더 본문에서 지역 변수로 한 번 계산**하고(`let theme = tokens(dark);`)
//!   클로저가 그 값을 **캡처**한다 — 아래 `Styled`. 토큰은 `Copy`라 비용도 없다.
//!
//! **주의**
//! - `Button::style`/`Button::resource_overrides`는 Reactor의 `Button`에만 있다
//!   (`generated.rs`). 다른 컨트롤은 `<Raw>`에서 WinUI 속성으로 조정한다.
//! - `ResourceOverrides`는 컨트롤의 `Resources` 사전에 값을 심는다 — 키는 WinUI 리소스
//!   이름(`ButtonBackground`/`ButtonBorderBrush`/`ButtonForeground`/`ButtonCornerRadius`)이고
//!   값은 `Color`/`Thickness`/`CornerRadius`만 지원한다. **컨트롤 하나만** 리테마할 때 쓴다
//!   (전역으로 바꾸려면 테마 브러시를 쓴다).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, Button, ButtonStyle, ChildrenControl, Color, Component, ComponentContext,
    ContentControl, CornerRadius, FontWeight, LayoutControl, Orientation, ResourceOverrides,
    StackPanel, TextBlock, ThemeBrush, Thickness, View, ViewContext, WindowBackdrop, WindowTheme,
    WindowVisuals,
};

/// 디자인 토큰 — 테마 하나가 정하는 값 묶음. **화면 코드는 이 값만 본다.**
#[derive(Clone, Copy, Debug, PartialEq)]
struct Tokens {
    /// 표면(카드/헤더)의 모서리 반지름 (DIP).
    radius: f64,
    /// 요소 사이 간격 (DIP).
    gap: f64,
    /// 카드 안쪽 여백 (DIP).
    pad: f64,
    /// 제목/본문 글자 크기 (DIP).
    title: f64,
    body: f64,
    /// 표면 색 — **테마 브러시**라 라이트/다크/고대비가 자동으로 따라온다.
    surface: Brush,
    /// 표면 테두리 색 — 같은 이유로 테마 브러시.
    stroke: Brush,
    /// 브랜드 색 — 테마 브러시 8종에 없으므로 **테마마다 직접** 준다.
    brand: Color,
    /// 브랜드 색 위에 얹는 글자색 — 브랜드 색이 밝으면 어둡게, 어두우면 밝게.
    on_brand: Color,
}

/// 테마 → 토큰. **순수 함수**라 헤드리스 테스트가 그대로 검증한다(아래 `tests`).
fn tokens(dark: bool) -> Tokens {
    Tokens {
        // 기하 토큰은 테마와 무관하게 고정 — "토큰"이 테마 전용 사전이 아님을 보여준다.
        radius: 8.0,
        gap: 12.0,
        pad: 16.0,
        title: 20.0,
        body: 12.0,
        // 테마를 따라가는 색: 브러시 이름만 주면 WinUI가 바꾼다.
        surface: Brush::from(ThemeBrush::CardBackground),
        stroke: Brush::from(ThemeBrush::CardStroke),
        // 테마별로 다른 색: 라이트에서는 진하게, 다크에서는 밝게.
        brand: if dark {
            Color::rgb(122, 193, 255)
        } else {
            Color::rgb(0, 95, 184)
        },
        on_brand: if dark {
            Color::rgb(0, 32, 64)
        } else {
            Color::rgb(255, 255, 255)
        },
    }
}

/// 헤더 — 토큰만 보고 만든 표면. 이 함수 안에 숫자도 색도 없다.
fn header(tokens: Tokens, dark: bool) -> View {
    let theme = if dark { "dark" } else { "light" };
    Border::new()
        // 표면 색은 테마 브러시 — 창 테마가 바뀌면 이 줄은 그대로여도 색이 바뀐다.
        .background(Brush::from(ThemeBrush::SolidBackground))
        .padding(Thickness::new(
            tokens.pad, tokens.gap, tokens.pad, tokens.gap,
        ))
        .content(
            StackPanel::new().spacing(tokens.gap / 3.0).children((
                TextBlock::new()
                    .text("디자인 토큰")
                    .font_size(tokens.title)
                    .font_weight(FontWeight::SEMI_BOLD),
                TextBlock::new()
                    .text(format!(
                        "테마: {theme} · radius {} · gap {}",
                        tokens.radius, tokens.gap
                    ))
                    .font_size(tokens.body)
                    // 보조 텍스트는 색을 하나 더 만들지 않고 opacity로 만든다.
                    .opacity(0.7),
            )),
        )
}

/// 카드 한 장 — 표면 + 테두리 + 모서리 + 여백. 토큰이 곧 스타일이다.
fn card(tokens: Tokens, title: &str, body: &str) -> View {
    Border::new()
        .background(tokens.surface)
        .border_brush(tokens.stroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(tokens.radius))
        .padding(Thickness::uniform(tokens.pad))
        .content(
            StackPanel::new().spacing(tokens.gap / 2.0).children((
                TextBlock::new()
                    .text(title)
                    .font_size(tokens.body + 2.0)
                    .font_weight(FontWeight::SEMI_BOLD),
                TextBlock::new()
                    .text(body)
                    .font_size(tokens.body)
                    .opacity(0.7),
            )),
        )
}

/// 카드 두 장을 나란히 — 같은 스타일 함수를 다른 내용으로 재사용한다.
fn card_row(tokens: Tokens) -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(tokens.gap)
        .children((
            card(
                tokens,
                "ThemeBrush::CardBackground",
                "테마 브러시 — 라이트/다크/고대비가 자동",
            ),
            card(tokens, "Tokens::brand", "브랜드 색 — 테마마다 직접 지정"),
        ))
}

/// WinUI가 이미 가진 컨트롤 스타일 — 직접 만들지 말고 **있는 걸 쓴다**.
fn button_styles() -> View {
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((
            Button::new().content("Default"),
            Button::new().style(ButtonStyle::Accent).content("Accent"),
            Button::new().style(ButtonStyle::Subtle).content("Subtle"),
            Button::new()
                .style(ButtonStyle::TextLink)
                .content("TextLink"),
        ))
}

/// 컨트롤 **하나만** 브랜드 색으로 — `Resources` 오버라이드.
///
/// 전역으로 바꾸려면 테마 브러시를 쓰는 편이 낫다(테마 전환을 따라가야 하므로).
fn branded_button(tokens: Tokens) -> View {
    let overrides = ResourceOverrides::new()
        .set("ButtonBackground", tokens.brand)
        .set("ButtonBorderBrush", tokens.brand)
        .set("ButtonForeground", tokens.on_brand)
        .set("ButtonCornerRadius", CornerRadius::uniform(tokens.radius));
    Button::new()
        .resource_overrides(overrides)
        .content("브랜드 버튼")
}

elm_magic::view! {
    fn Styled(dark = false, on_toggle: fn(bool)) {
        // 렌더 본문의 지역 변수 — `<Raw>` 클로저가 이 값을 **캡처**한다.
        // (`<Raw>` 본문은 매크로가 그대로 복사하므로 그 안에서 `dark`를 읽을 수 없다)
        let theme = tokens(dark);
        let is_dark = dark;

        <Col>
            // 스타일은 전부 Raw가 만든다 — 어댑터는 구조만 옮긴다(예제 16).
            <Raw>|out: &mut RawSlot| {
                *out = Some(header(theme, is_dark));
            }</Raw>

            <Raw>|out: &mut RawSlot| {
                *out = Some(card_row(theme));
            }</Raw>

            <Raw>|out: &mut RawSlot| {
                *out = Some(button_styles());
            }</Raw>

            <Raw>|out: &mut RawSlot| {
                *out = Some(branded_button(theme));
            }</Raw>

            // 전환 버튼은 elm이 그린다 — 호스트에게 "어느 테마로"를 콜백으로 알린다.
            <If when={is_dark}>
                <Text>"테마: dark"</Text>
                <Button on_click={on_toggle(false)}>"라이트로"</Button>
            <Else>
                <Text>"테마: light"</Text>
                <Button on_click={on_toggle(true)}>"다크로"</Button>
            </Else>
            </If>
        </Col>
    }
}

/// 호스트 — **테마 상태를 소유**하고 창 테마까지 책임진다.
struct Root {
    dark: bool,
}

#[derive(Clone)]
enum RootMessage {
    /// elm의 콜백 prop(`on_toggle`)이 큐에 넣는다.
    SetTheme(bool),
}

impl Component for Root {
    type Input = ();
    type Message = RootMessage;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        // 다크로 시작 — 토큰이 바꾸는 것이 색뿐임이 한눈에 보인다.
        Self { dark: true }
    }

    fn update(&mut self, message: RootMessage, _context: &ComponentContext<Self>) {
        match message {
            RootMessage::SetTheme(dark) => self.dark = dark,
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 디자인 토큰");
        // 창 테마(제목 표시줄 포함)는 호스트의 몫이다. `WindowVisuals`는 턴마다 비교되므로
        // 바뀐 값만 다시 적용된다 — `client_size`/`icon`/`WindowBackdrop`도 같은 자리다.
        context.window_visuals(
            WindowVisuals::new()
                .theme(if self.dark {
                    WindowTheme::Dark
                } else {
                    WindowTheme::Light
                })
                .backdrop(WindowBackdrop::Mica),
        );

        // elm → 호스트: 콜백 prop 안에서 자기 메시지 큐에 넣는다(예제 11).
        let sender = context.sender();
        let on_toggle = Callback::new(move |_arena, dark: bool| {
            let _ = sender.send(RootMessage::SetTheme(dark));
        });

        View::component::<ElmView<Styled>>(ElmInput::new(StyledProps {
            dark: Some(self.dark),
            on_toggle: Some(on_toggle),
            ..Default::default()
        }))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    /// props만 바꿔 계획을 뽑는다 — 호스트 없이도 화면 규칙을 검증한다.
    fn plan_for(dark: bool) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Styled>(
            &mut ctx,
            &StyledProps {
                dark: Some(dark),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 토큰 계약: **색만** 테마를 따르고, 표면은 테마 브러시라 값이 바뀌지 않는다.
    #[test]
    fn only_the_color_tokens_follow_the_theme() {
        let light = tokens(false);
        let dark = tokens(true);

        assert_ne!(light.brand, dark.brand, "브랜드 색은 테마마다 직접 준다");
        assert_ne!(
            light.on_brand, dark.on_brand,
            "글자색도 함께 바뀌어야 읽힌다"
        );

        assert_eq!(
            (light.radius, light.gap, light.pad, light.title, light.body),
            (dark.radius, dark.gap, dark.pad, dark.title, dark.body),
            "기하 토큰은 테마와 무관하다"
        );
        assert_eq!(
            light.surface, dark.surface,
            "테마 브러시는 값이 아니라 이름이다 — WinUI가 바꾼다"
        );
        assert_eq!(light.surface, Brush::from(ThemeBrush::CardBackground));
    }

    /// 구조 계약: 스타일 4묶음은 Raw가 만들고, 전환 버튼만 elm이 그린다.
    #[test]
    fn the_plan_has_one_raw_block_per_style_group() {
        let (node, pass) = plan_for(true);

        assert_eq!(pass.count("Raw"), 4, "헤더/카드/버튼 스타일/브랜드 버튼");
        assert_eq!(pass.count("Button"), 1, "전환 버튼만 elm 쪽이다");
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(
            kinds,
            vec!["Raw", "Raw", "Raw", "Raw", "TextBlock", "Button"],
            "`<If>`는 레이아웃 노드를 만들지 않는다"
        );
    }

    /// 분기는 호스트가 내려보낸 테마를 따른다.
    #[test]
    fn the_branch_follows_the_host_theme() {
        assert!(plan_for(true).1.has_text("테마: dark"));
        assert!(plan_for(false).1.has_text("테마: light"));
    }

    /// elm → 호스트: 버튼은 **바꾸려는 테마 값**을 콜백으로 넘긴다.
    #[test]
    fn the_toggle_asks_the_host_for_the_other_theme() {
        use std::cell::Cell;
        use std::rc::Rc;

        let asked = Rc::new(Cell::new(None));
        let seen = Rc::clone(&asked);
        let mut app = elm_magic::mount_with::<Styled>(StyledProps {
            dark: Some(false),
            on_toggle: Some(Callback::new(move |_arena, dark: bool| {
                seen.set(Some(dark));
            })),
            ..Default::default()
        });

        app.click("다크로");
        assert_eq!(asked.get(), Some(true), "라이트 화면은 다크를 요청한다");

        let mut app = elm_magic::mount_with::<Styled>(StyledProps {
            dark: Some(true),
            on_toggle: Some(Callback::new({
                let asked = Rc::clone(&asked);
                move |_arena, dark: bool| asked.set(Some(dark))
            })),
            ..Default::default()
        });
        app.click("라이트로");
        assert_eq!(asked.get(), Some(false), "다크 화면은 라이트를 요청한다");
    }
}
