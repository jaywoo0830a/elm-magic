//! gpui 예제 18 — 셸 레이아웃 (헤더 / 사이드바 / 본문 / 상태바)
//!
//! **언제 쓰나**: 앱의 뼈대를 잡을 때. gpui의 flex 레이아웃은 elm-magic의
//! `Col`/`Row` + `width/height: fill` + `flex-grow`로 대부분 표현된다.
//!
//! **매핑** (`crates/elm-magic-gpui/src/lib.rs`)
//! - `Col` → gpui `v_flex`, `Row` → `h_flex`
//! - `gap` / `row-gap` / `column-gap` → `gap` / `gap_y` / `gap_x`
//! - `padding`(+개별 변) → `p` / `px` / `py` …, `margin`은 gpui에 없어 **무시된다**
//! - `width/height: fill` → 부모가 주는 만큼, `flex-grow` → `flex_grow`
//! - `align` → `items_*`, `align-self` → `self_*`, `justify` → `justify_*`
//! - `overflow: hidden/scroll/auto` → `overflow_hidden()` (스크롤 컨테이너는 앱이 건다)
//! - `display: none` → 빈 `div()`(자리도 차지하지 않음), `visibility: hidden` → `invisible()`
//!
//! **베스트 패턴**
//! - 셸은 **고정 높이**(헤더 40, 상태바 24) + 본문 `flex-grow: 1` 조합이 가장 안정적이다.
//! - 창 크기에 따라 늘어나야 하는 곳에만 `fill`을 쓴다 — 남용하면 스크롤/오버플로가
//!   예측 불가능해진다.
//! - `margin`은 gpui에서 무시된다 → 간격은 `gap`/`padding`으로 표현한다.
//! - 스크롤이 필요하면 `<Raw>` + gpui `Window`로 스크롤 컨테이너를 감싸거나
//!   `overflow: hidden`만 걸고 본문을 작은 `ElmView`로 나눈다.
//!
//! **주의**: `Row` 안에서 자식이 넘치면 gpui는 기본적으로 줄바꿈하지 않는다 —
//! `wrap: true`를 명시하거나 `flex-shrink`를 조절한다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::css! {
    .shell { width: fill; height: fill; gap: 0; bg: background; }
    .topbar { height: 40; padding: 8 12; bg: surface; border-bottom-width: 1;
              border-color: border; align: center; gap: 8; }
    .main { flex-grow: 1; direction: row; gap: 0; }
    .sidebar { width: 200; padding: 12; gap: 6; bg: surface_alt; overflow: hidden; }
    .content { flex-grow: 1; padding: 16; gap: 8; }
    .statusbar { height: 24; padding: 4 12; bg: surface_alt; color: text_dim; font-size: 12; }
    .nav--active { bg: primary; color: on_primary; radius: 6; padding: 4 8; }
    .nav { padding: 4 8; radius: 6; }
}

elm_magic::view! {
    fn Sidebar(tab = 0) {
        <Col class="sidebar">
            <Text class="nav--active">"개요"</Text>
            <Button on_click={tab = 0}>"대시보드"</Button>
            <Button on_click={tab = 1}>"설정"</Button>
            <Button on_click={tab = 2}>"로그"</Button>
            "tab: {tab}"
        </Col>
    }
}

elm_magic::view! {
    fn Shell(tab = 0, online = true) {
        <Col class="shell">
            <Row class="topbar">
                <Strong>"elm-magic"</Strong>
                <Button on_click={online = !online}>"연결"</Button>
            </Row>
            <Row class="main">
                <Sidebar tab={tab} />
                <Col class="content">
                    <Text>"본문 — tab {tab}"</Text>
                    <Text class="muted">"여기서 fill/gap/overflow를 조절한다"</Text>
                </Col>
            </Row>
            <Row class="statusbar">
                "online: {online}"
            </Row>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(cx.new(ElmView::<Shell>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic::style::{Len, Palette};

    #[test]
    fn shell_uses_fill_and_grow() {
        let app = elm_magic::mount!(Shell);
        let shell = app.element().resolved_style(&Palette::dark());
        assert_eq!(shell.width, Some(Len::Fill));
        assert_eq!(shell.height, Some(Len::Fill));

        let main = app
            .element()
            .children()
            .and_then(|c| c.iter().find(|e| e.class().iter().any(|c| c == "main")))
            .expect(".main");
        assert_eq!(main.resolved_style(&Palette::dark()).flex_grow, Some(1.0));
    }

    #[test]
    fn sidebar_switch_changes_content() {
        let mut app = elm_magic::mount!(Shell);
        app.click("설정");
        app.assert_text("본문 — tab 1");
    }
}