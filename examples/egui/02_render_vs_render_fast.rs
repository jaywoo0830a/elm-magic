//! egui 예제 02 — `render` vs `render_fast` (앱 경로와 검증 경로)
//!
//! **언제 쓰나**: 렌더 루프에는 `render_fast`, 스타일/버튼이 실제로 전달됐는지
//! 확인할 때는 `render`.
//!
//! **왜 나뉘어 있나**: `render`는 노드마다 `ResolvedStyle`(수백 바이트)과 버튼 라벨
//! `String`을 복제해 `Pass`에 쌓는다. 참조 화면(1,000행 = 3,005노드)에서 프레임당 약
//! 2MB memcpy였다 — 그래서 앱 경로용 `render_fast`가 생겼다(0.8.3).
//! 그리는 그림은 **두 함수가 같다**(계약 테스트가 고정: `crates/elm-magic-egui/tests/adapter.rs`).
//!
//! **선택 기준**: `Pass`를 읽지 않으면 `render_fast`. 읽으면 `render`.

use elm_magic::prelude::*;

elm_magic::view! {
    fn Card(title = String::new()) {
        <Col class="card">
            <Text class="card__title">"{title}"</Text>
            {children}
        </Col>
    }
}

elm_magic::css! {
    .card { gap: 4; padding: 12; bg: surface; radius: 8; }
    .card__title { color: text_dim; }
}

fn ui(ui: &mut egui::Ui, ctx: &mut Ctx) {
    let tree = elm_magic::frame::<Card>(ctx, &CardProps::default());

    // ── (A) 앱 경로 ─────────────────────────────────────────
    elm_magic_egui::render_fast(ui, &tree, &mut ctx.arena);
}

fn ui_debug(ui: &mut egui::Ui, ctx: &mut Ctx) {
    let tree = elm_magic::frame::<Card>(ctx, &CardProps::default());

    // ── (B) 검증/디버깅 경로 ─────────────────────────────────
    let pass = elm_magic_egui::render(ui, &tree, &mut ctx.arena);

    // 스타일이 코어 해석대로 전달됐는지 egui 내부를 들여다보지 않고 확인한다.
    let col = pass.style_of("col").expect("Col 스타일");
    assert_eq!(col.gap, Some(4.0));
    assert_eq!(col.padding, Some(elm_magic::style::Edges::splat(12.0)));

    // 그려진 버튼의 실제 rect — 클릭 시뮬레이션 좌표로 쓴다(예제 18).
    for (label, response) in &pass.buttons {
        println!("button {label:?} at {:?}", response.rect);
    }
}
