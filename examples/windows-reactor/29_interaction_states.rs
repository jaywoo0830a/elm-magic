//! windows-reactor 예제 29 — **상태 표현과 전환**: 정상/로딩/오류/비활성 + 부드러운 전환
//! **업스트림 대응**: `opacity-transition` · `scale-transition` · `exit-transition` ·
//! `theme-transition` — 업스트림도 전환을 **컨트롤 속성**으로 선언하고 값이 바뀔 때 WinUI가
//! 보간하게 둔다(애니메이션 루프를 돌리지 않는다).
//!
//!
//! **언제 쓰나**: 같은 화면이 상태에 따라 다르게 보여야 할 때(로딩/오류/비활성).
//!
//! **상태를 색으로 직접 칠하지 않는다** — 상태 → 시각 사양을 함수로 둔다([`state_style`]).
//! - 정상: 테마 기본(`CardBackground`/`CardStroke`) + 브랜드 색 테두리.
//! - 로딩: `ProgressBar::is_indeterminate` + **opacity**로 "아직 아님"을 표현한다.
//! - 오류: `ProgressBar::show_error` + 오류 색 + `InfoBar`(severity Error).
//! - 비활성: `is_enabled(false)` — **회색 표현은 WinUI가 만든다**(직접 칠하면 고대비에서 깨진다).
//!
//! **전환(transition)은 "상태가 바뀔 때 부드럽게"를 담당한다**
//! - `Border::{scale, scale_transition(Duration), opacity_transition(Duration)}` —
//!   값이 바뀌면 WinUI가 그 시간 동안 보간한다. **elm 상태를 바꾸는 것만으로 애니메이션이
//!   생긴다** — 우리가 애니메이션 루프를 돌리지 않는다(elm에는 시간 개념이 없다).
//! - `Duration`은 `std::time::Duration`이다(`Duration::from_millis(160)`).
//! - Reactor 0.100의 전환은 **opacity/scale 두 가지뿐**이다(위치 이동 전환은 없다).
//!
//! **주의**
//! - `scale`은 **레이아웃 크기를 바꾸지 않는다**(그려진 결과만 커진다) — 옆 요소가 밀리지 않는다.
//! - 상태는 **하나**다(로딩 + 오류를 동시에 두지 않는다). 그래서 elm 상태도 정수 하나로 두고
//!   [`state_from_index`]가 해석한다 — 화면 코드가 분기로 지저분해지지 않는다.
//! - 전환 시간은 **짧게**(120~200ms). 길면 "느린 앱"으로 느껴진다.

use std::time::Duration;

use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, Button, ChildrenControl, Color, Component, ComponentContext,
    ContentControl, CornerRadius, FontWeight, InfoBar, InfoBarSeverity, KeyedView, LayoutControl,
    ProgressBar, StackPanel, TextBlock, ThemeBrush, Thickness, View, ViewContext,
};

/// 화면 상태 — 한 번에 하나만 참이다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Loading,
    Error,
    Disabled,
}

impl State {
    /// 상태 순서 — 버튼과 테스트가 같은 순서를 본다.
    /// 화면은 인덱스를 직접 쓰고, 이 목록은 `#[cfg(test)]`가 "모든 상태 쌍"을
    /// 훑을 때 쓴다 — `allow`는 "테스트 전용 계약"이라는 표시다.
    #[allow(dead_code)]
    const ALL: [State; 4] = [State::Idle, State::Loading, State::Error, State::Disabled];

    fn label(self) -> &'static str {
        match self {
            State::Idle => "정상",
            State::Loading => "로딩",
            State::Error => "오류",
            State::Disabled => "비활성",
        }
    }
}

/// elm 상태(정수 하나) → 화면 상태. 모르는 값은 정상으로 본다.
fn state_from_index(index: i32) -> State {
    match index {
        1 => State::Loading,
        2 => State::Error,
        3 => State::Disabled,
        _ => State::Idle,
    }
}

/// 상태 → 시각 사양. **순수 데이터**라 테스트가 그대로 검증한다.
#[derive(Clone, Copy, Debug, PartialEq)]
struct StateStyle {
    /// 표면 불투명도 — "아직 준비되지 않음"을 색이 아니라 투명도로 표현한다.
    opacity: f64,
    /// 테두리/강조 색.
    accent: Color,
    /// 조작 가능한가(`is_enabled`로 그대로 넘긴다).
    interactive: bool,
}

fn state_style(state: State) -> StateStyle {
    match state {
        State::Idle => StateStyle {
            opacity: 1.0,
            accent: Color::rgb(0, 95, 184),
            interactive: true,
        },
        State::Loading => StateStyle {
            opacity: 0.6,
            accent: Color::rgb(0, 95, 184),
            interactive: false,
        },
        State::Error => StateStyle {
            opacity: 1.0,
            accent: Color::rgb(196, 43, 28),
            interactive: true,
        },
        State::Disabled => StateStyle {
            opacity: 0.5,
            accent: Color::rgb(128, 128, 128),
            interactive: false,
        },
    }
}

/// 상태 카드 — 사양대로 표면을 만들고, 진행 표시와 버튼을 곁들인다.
fn state_card(state: State) -> View {
    let style = state_style(state);

    // 진행 표시는 상태가 정한다(직접 만들지 않는다).
    let indicator: Option<View> = match state {
        State::Loading => Some(ProgressBar::new().is_indeterminate(true).into()),
        State::Error => Some(ProgressBar::new().value(60.0).show_error(true).into()),
        _ => None,
    };

    let mut children: Vec<View> = vec![
        TextBlock::new()
            .text(format!("상태: {}", state.label()))
            .font_size(13.0)
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
        TextBlock::new()
            .text(format!(
                "opacity {} · interactive {}",
                style.opacity, style.interactive
            ))
            .font_size(11.0)
            .opacity(0.6)
            .into(),
    ];
    if let Some(indicator) = indicator {
        children.push(indicator);
    }
    // 비활성 표현은 WinUI가 만든다 — 우리는 `is_enabled`만 준다.
    children.push(
        Button::new()
            .is_enabled(style.interactive)
            .content("동작 실행"),
    );

    let body = StackPanel::new().spacing(6.0).keyed_children(
        children
            .into_iter()
            .enumerate()
            .map(|(index, view)| KeyedView::new(index as u64, view)),
    );

    Border::new()
        .width(280.0)
        .background(Brush::from(ThemeBrush::CardBackground))
        .border_brush(style.accent)
        .border_thickness(Thickness::uniform(2.0))
        .corner_radius(CornerRadius::uniform(6.0))
        .padding(Thickness::uniform(12.0))
        // 상태가 바뀌면 이 값이 전환 시간 동안 보간된다.
        .opacity(style.opacity)
        .opacity_transition(Duration::from_millis(180))
        .content(body)
}

/// 심각도 배너 — `InfoBar`가 색·아이콘·닫기 버튼을 담당한다(직접 만들지 않는다).
fn severity_banner(state: State) -> View {
    let style = state_style(state);
    let severity = match state {
        State::Error => InfoBarSeverity::Error,
        State::Loading => InfoBarSeverity::Informational,
        _ => InfoBarSeverity::Success,
    };

    InfoBar::new()
        .severity(severity)
        .title(format!("상태: {}", state.label()))
        .message(format!(
            "accent rgb({}, {}, {})",
            style.accent.r, style.accent.g, style.accent.b
        ))
        .is_open(true)
        // `InfoBar`는 `View`가 아니라 `Into<View>`다 — 마지막에 옮긴다.
        .into()
}

/// 확대 토글 — 전환 시간을 주면 **상태 변경만으로** 애니메이션이 생긴다.
fn zoom_card(zoomed: bool) -> View {
    Border::new()
        .width(200.0)
        .height(72.0)
        .background(Brush::from(ThemeBrush::Accent))
        .corner_radius(CornerRadius::uniform(6.0))
        .padding(Thickness::uniform(10.0))
        // 값이 바뀌면 이 시간 동안 보간된다 — 우리는 애니메이션을 돌리지 않는다.
        .scale(if zoomed { 1.15 } else { 1.0 })
        .scale_transition(Duration::from_millis(160))
        .content(
            TextBlock::new()
                .text(if zoomed {
                    "확대됨 (1.15×)"
                } else {
                    "기본 (1.0×)"
                })
                .font_size(12.0)
                .foreground(Color::rgb(255, 255, 255)),
        )
}

elm_magic::view! {
    fn States(state = 0, zoomed = false) {
        // `<Raw>`가 캡처할 값은 렌더 본문에서 계산한다(예제 21).
        let current = state_from_index(state);
        let zoom = zoomed;

        <Col>
            <Raw>|out: &mut RawSlot| {
                *out = Some(state_card(current));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(severity_banner(current));
            }</Raw>
            <Raw>|out: &mut RawSlot| {
                *out = Some(zoom_card(zoom));
            }</Raw>

            <Button on_click={state = 0}>"정상으로"</Button>
            <Button on_click={state = 1}>"로딩으로"</Button>
            <Button on_click={state = 2}>"오류로"</Button>
            <Button on_click={state = 3}>"비활성으로"</Button>

            <Switch on={state}>
                <Case when={0}><Text>"상태: 정상"</Text></Case>
                <Case when={1}><Text>"상태: 로딩"</Text></Case>
                <Case when={2}><Text>"상태: 오류"</Text></Case>
                <Default><Text>"상태: 비활성"</Text></Default>
            </Switch>

            <If when={zoomed}>
                <Text>"확대: on"</Text>
                <Button on_click={zoomed = false}>"축소하기"</Button>
            <Else>
                <Text>"확대: off"</Text>
                <Button on_click={zoomed = true}>"확대하기"</Button>
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
        context.window_title("elm-magic — 상태 표현");
        View::component::<ElmView<States>>(ElmInput::new(StatesProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::Ctx; // `State`는 이 파일이 직접 정의한다 — 전역 import는 모호해진다
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    fn plan_for(state: i32) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<States>(
            &mut ctx,
            &StatesProps {
                state: Some(state),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 인덱스 → 상태: 네 상태를 모두 덮고, 모르는 값은 정상이다.
    #[test]
    fn the_index_maps_to_all_four_states() {
        assert_eq!(state_from_index(0), State::Idle);
        assert_eq!(state_from_index(1), State::Loading);
        assert_eq!(state_from_index(2), State::Error);
        assert_eq!(state_from_index(3), State::Disabled);
        assert_eq!(state_from_index(99), State::Idle, "모르는 값은 정상");
    }

    /// 로딩/비활성은 표면이 흐려지고, 오류는 **또렷해야** 한다.
    #[test]
    fn loading_and_disabled_dim_the_surface() {
        assert!(state_style(State::Loading).opacity < 1.0);
        assert!(state_style(State::Disabled).opacity < 1.0);
        assert_eq!(state_style(State::Idle).opacity, 1.0);
        assert_eq!(state_style(State::Error).opacity, 1.0);
    }

    /// 조작 가능 여부는 상태가 정한다(화면이 직접 판단하지 않는다).
    #[test]
    fn only_idle_and_error_stay_interactive() {
        assert!(state_style(State::Idle).interactive);
        assert!(
            state_style(State::Error).interactive,
            "오류는 재시도할 수 있어야 한다"
        );
        assert!(!state_style(State::Loading).interactive);
        assert!(!state_style(State::Disabled).interactive);
    }

    /// 네 상태는 서로 다른 사양이다(표가 실제로 갈라져 있다).
    #[test]
    fn the_four_states_are_distinct() {
        for (index, state) in State::ALL.iter().enumerate() {
            for other in State::ALL.iter().skip(index + 1) {
                assert_ne!(
                    state_style(*state),
                    state_style(*other),
                    "{state:?}와 {other:?}의 사양이 같다"
                );
            }
        }
    }

    /// 구조: 상태 카드/배너/확대 카드 세 묶음은 Raw, 버튼 다섯과 라벨은 elm.
    #[test]
    fn the_plan_has_three_raw_blocks_and_five_buttons() {
        let (node, pass) = plan_for(0);
        assert_eq!(pass.count("Raw"), 3);
        assert_eq!(pass.count("Button"), 5, "상태 4 + 확대 토글 1");
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(
            kinds,
            vec![
                "Raw",
                "Raw",
                "Raw",
                "Button",
                "Button",
                "Button",
                "Button",
                "TextBlock",
                "TextBlock",
                "Button"
            ],
            "`<Switch>`/`<If>`는 레이아웃 노드를 만들지 않는다"
        );
    }

    /// 상태 버튼이 화면 라벨을 바꾼다(헤드리스에서 클릭까지).
    #[test]
    fn the_state_buttons_flip_the_label() {
        let mut app = elm_magic::mount!(States);
        app.assert_text("상태: 정상");
        app.click("로딩으로");
        app.assert_text("상태: 로딩");
        app.click("오류로");
        app.assert_text("상태: 오류");
        app.click("비활성으로");
        app.assert_text("상태: 비활성");
        app.click("정상으로");
        app.assert_text("상태: 정상");
    }

    /// 확대 토글도 같은 방식으로 확인한다.
    #[test]
    fn the_zoom_toggle_flips_the_label() {
        let mut app = elm_magic::mount!(States);
        app.assert_text("확대: off");
        app.click("확대하기");
        app.assert_text("확대: on");
        app.click("축소하기");
        app.assert_text("확대: off");
    }
}
