//! windows-reactor 예제 18 — 셸 레이아웃 (헤더 / 사이드바 / 본문 / 상태바)
//!
//! **언제 쓰나**: 앱의 뼈대를 잡을 때.
//!
//! **기본 대응**
//! - `<Col>` → `StackPanel(Orientation::Vertical)`, `<Row>` → `Horizontal`.
//! - 어댑터는 **spacing을 넣지 않는다**(스타일 계층이 없다) — WinUI 기본값 0이다.
//!   간격이 필요하면 `<Raw>`로 `StackPanel::spacing`을 준다(예제 16).
//! - 창 크기에 따라 늘어나는 영역은 WinUI에서 `Grid`의 `Star` 행/열로 표현한다.
//!   2D 레이아웃은 `<Raw>`에서 `Grid::new().rows([..]).columns([..])`로 만든다
//!   (`GridLength::{Auto, Pixel(f64), Star(f64)}`, `GridLength::STAR`).
//! - 스크롤이 필요하면 `<Raw>`에서 `ScrollViewer`를 쓴다.
//!
//! **베스트 패턴**
//! - **셸은 한 곳에** 모으고(헤더/사이드바/상태바), 내용만 `ElmView` 여러 개로 나눈다
//!   (예제 04). 그러면 내용 갱신이 셸을 다시 그리지 않는다.
//! - 셸의 구성 요소(현재 탭, 온라인 여부)는 셸 컴포넌트의 상태로 두고,
//!   필요하면 props로 내려보낸다(예제 11).
//! - 고정 높이가 필요한 줄(헤더/상태바)은 `Border::height` + `LayoutControl`로 잡는다.
//!
//! **주의**: `margin` 같은 CSS 개념은 이 백엔드에 없다 — 여백은 `Border::padding`/
//! `Thickness`, 간격은 `StackPanel::spacing`이 담당한다.
//!
//! **자식은 슬롯을 직접 쓰지 않는다**: `on_click={tab = 1}`을 자식 안에서 쓰면
//! 그때부터 그 슬롯은 **자식의 상태**가 되어 props를 따라가지 않는다(`slot_prop`의
//! prop 동기화는 자식이 아직 안 쓴 동안만 유효하다). 공유 값은 **셸이 소유**하고
//! 자식에게는 props + 콜백(`on_select: fn(i32)`)을 내려보낸다 — 아래 `Sidebar`.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, ChildrenControl, Component, ComponentContext, ContentControl, LayoutControl,
    Orientation, StackPanel, TextBlock, ThemeBrush, Thickness, View, ViewContext,
};

elm_magic::view! {
    fn Sidebar(tab = 0, on_select: fn(i32)) {
        <Col>
            <Button on_click={on_select(0)}>"대시보드"</Button>
            <Button on_click={on_select(1)}>"설정"</Button>
            <Button on_click={on_select(2)}>"로그"</Button>
            "tab: {tab}"
        </Col>
    }
}

elm_magic::view! {
    fn Content(tab = 0) {
        <Col>
            <Strong>"본문"</Strong>
            "tab {tab}의 내용"
        </Col>
    }
}

elm_magic::view! {
    fn Shell(tab = 0, online = true) {
        <Col>
            // 헤더 — 높이와 배경은 WinUI가 정한다.
            <Raw>|out: &mut RawSlot| {
                let bar = Border::new()
                    .height(48.0)
                    .padding(Thickness::new(12.0, 0.0, 12.0, 0.0))
                    .background(Brush::from(ThemeBrush::SolidBackground))
                    .content(TextBlock::new().text("elm-magic"));
                *out = Some(bar);
            }</Raw>

            <Row>
                <Sidebar tab={tab} on_select={tab = _} />
                <Content tab={tab} />
            </Row>

            <Raw>|out: &mut RawSlot| {
                let status = StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((TextBlock::new().text("statusbar").font_size(12.0),));
                *out = Some(status);
            }</Raw>
            "online: {online}"
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
        context.window_title("elm-magic — 셸");
        View::component::<ElmView<Shell>>(ElmInput::new(ShellProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn shell_structure_is_predictable() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Shell>(&mut ctx, &ShellProps::default());
        let (node, pass) = plan(&tree);

        // Raw(헤더) / Row(사이드바+본문) / Raw(상태바) / online 텍스트
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(
            kinds,
            vec!["Raw", "StackPanel", "Raw", "TextBlock"],
            "셸의 골격이 고정돼야 회귀를 잡을 수 있다"
        );
        assert_eq!(pass.count("Raw"), 2);
        assert!(pass.has_text("online: true"));
    }

    #[test]
    fn sidebar_navigation_updates_the_body() {
        let mut app = elm_magic::mount!(Shell);
        app.click("설정");
        app.assert_text("tab 1의 내용");
        app.assert_text("tab: 1");
    }
}
