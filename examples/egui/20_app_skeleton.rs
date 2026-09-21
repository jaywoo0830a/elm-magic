//! egui 예제 20 — 실전 골격 (main + Ctx + 스타일 + 전역 상태 + 테마)
//!
//! **언제 쓰나**: 새 앱을 시작할 때 그대로 복사하는 뼈대. 지금까지의 조각을
//! 한 파일에 모았다.
//!
//! **구성**
//! 1. `install_styles()` — `css!`는 시작 섹션에서 `style::register`를 대신 불러 주는
//!    매크로다. `wasm`처럼 시작 섹션이 안 도는 환경이거나 매크로를 쓰고 싶지 않으면
//!    아래처럼 **직접 호출**하면 된다(`examples/gpui/16` 참고).
//! 2. `#[store] App` — 테마/세션 같은 전역 진실.
//! 3. `Root` 컴포넌트 — 셸(헤더/본문/상태바)만 담당하고, 화면은 자식에게 맡긴다.
//! 4. `main()` — `Ctx` 소유, 테마 선택, `frame` → `render_fast`.
//!
//! **베스트 패턴**
//! - `Ctx`는 **하나만** 만든다. 화면마다 `Ctx`를 새로 만들면 전역 상태가 갈라진다.
//! - 매 프레임 하는 일은 `frame` + `render_fast` 두 줄로 고정한다(그 밖의 코드는
//!   프레임 바깥으로 밀어낸다).
//! - 스타일은 앱 시작 때 **한 번** 등록하고, 팔레트만 테마에 따라 갈아 끼운다.
//! - 창 크기가 바뀌어도 레이아웃이 견디도록 `width: fill`/`height: fill`을 쓴다.
//!
//! **주의**: `css!`는 **항목 위치(item position)** 에 쓴다(함수 안이면 컴파일 에러).
//! 스타일을 런타임 값에 따라 바꾸고 싶다면 `style::register`를 직접 부른다.

use elm_magic::prelude::*;
use elm_magic::style::{Edges, StyleSpec, Token};

// ── 1. 스타일 ───────────────────────────────────────────────
elm_magic::css! {
    .app       { width: fill; height: fill; gap: 0; bg: background; }
    .app__bar  { height: 40; padding: 8 12; bg: surface; border-bottom-width: 1;
                 border-color: border; align: center; }
    .app__body { padding: 12; gap: 8; flex-grow: 1; }
    .app__status { height: 24; padding: 4 12; bg: surface_alt; color: text_dim; font-size: 12; }
    .card      { padding: 12; gap: 8; bg: surface; radius: 8; shadow: 0 2 8 shadow; }
    .muted     { color: text_dim; }
}

// ── 2. 전역 상태 ────────────────────────────────────────────
#[store]
struct App {
    dark: bool,
}

// ── 3. 화면 ────────────────────────────────────────────────
elm_magic::view! {
    fn Header() {
        <Row class="app__bar">
            <Strong>"elm-magic"</Strong>
            <Button on_click={app.dark = !app.dark}>"테마"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn Counter(n = 0) {
        <Col class="card">
            "Count: {n}"
            <Row>
                <Button on_click={n -= 1}>"-"</Button>
                <Button on_click={n += 1}>"+"</Button>
            </Row>
        </Col>
    }
}

elm_magic::view! {
    fn Root() {
        <Col class="app">
            <Header />
            <Col class="app__body">
                <Counter />
                <Text class="muted">"Ctrl+S 같은 단축키는 on_key로"</Text>
            </Col>
            <Row class="app__status">
                "dark: {app.dark}"
            </Row>
        </Col>
    }
}

// ── 4. main ────────────────────────────────────────────────
fn main() -> eframe::Result<()> {
    // 상태 아레나 + 전역 상태는 앱 수명 동안 하나만 존재한다.
    let mut ctx = Ctx::default();

    eframe::run_simple_native(
        "elm-magic app",
        eframe::NativeOptions::default(),
        move |egui_ctx, _frame| {
            // 테마 전환은 두 갈래다.
            // (a) egui 쪽 설정을 바꾼다 → 어댑터가 Visuals::dark_mode를 보고 팔레트를 고른다.
            //     (설정 UI가 egui라면 여기서 `egui_ctx.set_visuals(..)`를 부른다.)
            // (b) 앱 팔레트를 직접 골라 `render_with_palette`로 넘긴다(예제 03).
            // 지금은 (a) 기본값을 그대로 쓴다.
            let _ = egui_ctx;

            egui::CentralPanel::default().show(egui_ctx, |ui| {
                let tree = elm_magic::frame::<Root>(&mut ctx, &RootProps::default());
                elm_magic_egui::render_fast(ui, &tree, &mut ctx.arena);
            });
        },
    )
}

/// `css!` 없이 스타일을 등록하는 동등한 코드 (wasm/동적 스타일용).
#[allow(dead_code)]
fn install_styles() {
    elm_magic::style::register(vec![
        (
            ".card",
            StyleSpec {
                gap: Some(8.0),
                padding: Some(Edges::splat(12.0)),
                bg: Some(Token::Surface),
                radius: Some(8.0),
                ..StyleSpec::NONE
            },
        ),
        (
            ".card:hover",
            StyleSpec { bg: Some(Token::SurfaceAlt), ..StyleSpec::NONE },
        ),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_renders_head_body_status() {
        let app = elm_magic::mount!(Root);
        app.assert_text("elm-magic");
        app.assert_text("Count: 0");
        app.assert_text("dark: false");
    }

    #[test]
    fn styles_are_registered_for_the_shell() {
        let app = elm_magic::mount!(Root);
        let col = app.element().resolved_style(&elm_magic::style::Palette::dark());
        assert!(col.width.is_some(), "`.app`의 width: fill이 해석돼야 한다");
    }
}
