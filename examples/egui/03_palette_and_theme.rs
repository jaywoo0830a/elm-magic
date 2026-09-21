//! egui 예제 03 — 팔레트/테마 (`render_with_palette`)
//!
//! **언제 쓰나**: 라이트/다크를 egui 설정이 아니라 **앱이** 고르고 싶을 때,
//! 또는 브랜드 색을 토큰에 심고 싶을 때.
//!
//! **핵심 계약**: "테마는 팔레트다" (사양서 6.3). `css!`에 쓴 `bg: surface`는
//! 팔레트의 `Token::Surface`로 해석된다 — 팔레트를 바꾸면 스타일 파일을 고치지 않고
//! 색이 따라온다. `render`/`render_fast`는 `ui.visuals().dark_mode`로 팔레트를
//! 자동 선택하고, `render_with_palette`는 그 선택을 호출자에게 넘긴다.
//!
//! **주의**: 팔레트는 **한 프레임 동안 고정**되어야 한다. 렌더 도중에 바꾸면
//! 형제 노드가 서로 다른 색을 본다.

use elm_magic::prelude::*;
use elm_magic::style::{Color, Palette, Token};

elm_magic::view! {
    fn Badge(label: String, tone = String::from("info")) {
        <Row class="badge">
            <Text class="badge__dot">"●"</Text>
            "{label} ({tone})"
        </Row>
    }
}

elm_magic::css! {
    .badge      { gap: 6; padding: 6 10; bg: surface; radius: 999; }
    .badge__dot { color: primary; }
}

/// `css!` 토큰 14종 중 브랜드 색만 바꾼 팔레트.
fn brand_palette(dark: bool) -> Palette {
    let mut p = if dark { Palette::dark() } else { Palette::light() };
    // `Color::rgba` — 팔레트에 직접 넣은 색은 `css!`의 모든 `primary`에 반영된다.
    p.set(Token::Primary, Color::rgba(0x6c, 0x5c, 0xe7, 255));
    p.set(Token::OnPrimary, Color::rgba(255, 255, 255, 255));
    p
}

fn badge_ui(ui: &mut egui::Ui, ctx: &mut Ctx, props: &BadgeProps) {
    let tree = elm_magic::frame::<Badge>(ctx, props);

    // egui 설정을 따르는 기본 경로
    let palette = if ui.visuals().dark_mode {
        brand_palette(true)
    } else {
        brand_palette(false)
    };
    elm_magic_egui::render_with_palette(ui, &tree, &mut ctx.arena, &palette);
}

/// 앱이 테마를 직접 관리한다면(예: 사용자가 라이트 모드를 강제) 이렇게 한다.
fn forced_light(ui: &mut egui::Ui, ctx: &mut Ctx) {
    let tree = elm_magic::frame::<Badge>(ctx, &BadgeProps::default());
    let light = Palette::light();
    elm_magic_egui::render_with_palette(ui, &tree, &mut ctx.arena, &light);
}
