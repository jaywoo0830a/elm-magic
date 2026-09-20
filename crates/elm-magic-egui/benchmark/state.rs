//! 상태 추적 게이트 — `needs_pointer_state()`가 **양쪽으로** 옳은가 (0.8.3).
//!
//! `:hover`/`:active`/`:focus`를 요구하는 규칙이 하나도 없으면 어댑터는 노드마다 하는
//! 상호작용 상태 읽기/쓰기(egui temp data 왕복 + 컨텍스트 락)를 건너뛴다 — 실측
//! 노드당 37ns(`benchmark_hotspots` H6)에 락이 더해진다. 이 게이트는 "그림이 바뀌지
//! 않을 때만" 켜져야 하므로 **없으면 안 읽고, 있으면 읽는다**를 둘 다 고정한다.
//!
//! 전역 레지스트리를 비우고 다시 채우므로 **단독 프로세스**여야 한다(자기 파일 +
//! `[[test]]`). 그리고 두 방향을 한 함수에서 순서대로 검사한다 — 같은 프로세스 안에서
//! 테스트가 병렬로 돌면 레지스트리를 서로 덮어쓴다.

#[path = "harness.rs"]
mod harness;

use elm_magic::prelude::*;
use elm_magic::style::StyleSpec;
use harness::*;

/// 트리를 `frames`번 그린 뒤 egui temp data 항목 수를 돌려준다.
fn temp_entries(egui_ctx: &egui::Context, frames: usize) -> usize {
    let mut app = mount_with::<BenchList>(BenchListProps {
        items: Some(items(20)),
        ..Default::default()
    });
    let tree = app.element().clone();
    let arena = &mut app.ctx.arena;
    for _ in 0..frames {
        let mut out = egui_ctx.run_ui(raw_input(), |ui| {
            elm_magic_egui::render_fast(ui, &tree, arena);
        });
        out.textures_delta.clear();
    }
    egui_ctx.data(|d| d.len())
}

#[test]
fn pointer_state_tracking_follows_the_rules() {
    // ── 1) 상태를 보는 규칙이 없으면 추적하지 않는다 ─────────────
    elm_magic::style::reset_for_tests();
    assert!(
        !elm_magic::style::needs_pointer_state(),
        "규칙이 없으면 `:hover`를 볼 일도 없다"
    );
    let egui_ctx = egui::Context::default();
    let quiet = temp_entries(&egui_ctx, 5);
    assert_eq!(
        quiet, 0,
        "상태 추적이 꺼졌는데 egui temp data가 생겼다 (노드 메모리 낭비)"
    );

    // ── 2) `:hover` 규칙이 생기면 추적한다 ───────────────────────
    elm_magic::style::register(vec![(
        ".bench_row:hover",
        StyleSpec {
            opacity: Some(0.5),
            ..Default::default()
        },
    )]);
    assert!(
        elm_magic::style::needs_pointer_state(),
        "`:hover` 규칙이 있으면 상태를 읽어야 한다"
    );

    // 새 컨텍스트로 다시 그린다 (앞의 컨텍스트에는 노드 메모리가 없다).
    let egui_ctx = egui::Context::default();
    let nodes = {
        let app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(20)),
            ..Default::default()
        });
        node_count(app.element())
    };
    let tracked = temp_entries(&egui_ctx, 5);
    assert!(
        tracked >= nodes,
        "상태 추적이 켜졌는데 노드 메모리가 없다: 항목 {tracked}개 < 노드 {nodes}개"
    );
    // 프레임을 반복해도 늘지 않는다 (노드 순번 키를 매 프레임 덮어쓴다).
    assert_eq!(
        tracked,
        temp_entries(&egui_ctx, 5),
        "프레임 반복으로 노드 메모리가 늘었다"
    );
}
