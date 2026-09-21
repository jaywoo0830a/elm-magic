//! egui 예제 15 — `Pass`로 스타일 전달 검증 (코어 해석 ↔ 어댑터 패리티)
//!
//! **언제 쓰나**: "왜 이 컴포넌트만 스타일이 안 먹지?"를 **egui 내부를 뒤지지 않고**
//! 확인할 때. CI에서 스타일 회귀를 잡는 가장 싼 방법이다.
//!
//! **계약**
//! - 코어는 플랫폼 독립적으로 `ResolvedStyle`을 확정한다 —
//!   `element.resolved_style(&palette)` / `resolved_style_in(조상, 상태, 상속, 팔레트)`.
//! - 어댑터는 그 결과를 **옮기기만** 하고, `render`(=`render_fast` 아님)가
//!   `Pass::styles`에 `(태그, 확정 스타일)`을 남긴다.
//! - 그래서 두 값을 **직접 비교**할 수 있다 → 어댑터가 속이면 즉시 드러난다.
//!   (실제 계약 테스트: `crates/elm-magic-egui/tests/adapter.rs::egui_styles_match_headless_resolution`)
//!
//! **베스트 패턴**: 헤드리스에서 기대값을 먼저 박고(예: `gap == 8`), 같은 기대값을
//! `Pass`로 한 번 더 확인한다. 스타일 버그는 대개 이 두 값이 갈라지는 지점에서 시작한다.
//!
//! **주의**: `Pass::styles`는 "스타일이 **비어 있지 않은**" 노드만 담는다.
//! 선언이 하나도 없는 노드(들여쓰기용 빈 `Col` 등)는 목록에 없다.

use elm_magic::prelude::*;
use elm_magic::style::Palette;

elm_magic::css! {
    .panel { gap: 8; padding: 16; bg: surface; radius: 8; }
    .panel__title { color: text_dim; font-size: 18; weight: bold; }
    .panel Button { radius: 6; }
}

elm_magic::view! {
    fn Panel(n = 0) {
        <Col class="panel">
            <Text class="panel__title">"hello"</Text>
            <Button on_click={n += 1}>"ok {n}"</Button>
        </Col>
    }
}

/// egui 한 프레임을 그리고 `Pass`를 돌려준다.
fn frame(ui: &mut egui::Ui, ctx: &mut Ctx) -> elm_magic_egui::Pass {
    let tree = elm_magic::frame::<Panel>(ctx, &PanelProps::default());
    elm_magic_egui::render(ui, &tree, &mut ctx.arena)
}

/// 진단: 코어 해석과 어댑터 기록을 나란히 찍는다.
fn dump(ui: &mut egui::Ui, ctx: &mut Ctx) {
    let tree = elm_magic::frame::<Panel>(ctx, &PanelProps::default());
    let headless = tree.resolved_style(&Palette::dark());
    let pass = elm_magic_egui::render(ui, &tree, &mut ctx.arena);

    let egui_side = pass.style_of("col").expect("Col 스타일이 전달됐다");
    assert_eq!(headless.gap, egui_side.gap, "gap이 코어와 다르다");
    assert_eq!(headless.padding, egui_side.padding);
    assert_eq!(headless.bg, egui_side.bg);

    for (tag, style) in &pass.styles {
        println!("{tag}: gap={:?} padding={:?}", style.gap, style.padding);
    }
    for (label, response) in &pass.buttons {
        println!("button {label:?} rect={:?}", response.rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 헤드리스로 기대값을 못 박는다 — egui 없이 CI에서 가장 먼저 도는 계약.
    #[test]
    fn core_resolution_is_the_source_of_truth() {
        let app = elm_magic::mount!(Panel);
        let col = app.element().resolved_style(&Palette::dark());
        assert_eq!(col.gap, Some(8.0));
        assert_eq!(col.padding, Some(elm_magic::style::Edges::splat(16.0)));
        assert_eq!(col.radius, Some(8.0));

        let title = app
            .element()
            .children()
            .and_then(|c| c.iter().find(|e| e.tag() == "text"))
            .expect("Text 자식")
            .resolved_style(&Palette::dark());
        assert_eq!(title.bold, Some(true));
    }
}
