//! 회귀 가드 — 벽시계에 기대지 않는 **불변식**으로 먼저 잡고, 시간 예산은 옵트인.
//!
//! 벽시계 단언은 CI에서 flaky하다(이 저장소의 `tests/resource_budget.rs`도 같은 이유로
//! 시간을 단언하지 않는다). 그래서 이 파일은:
//!
//! 1. **구조 불변식**을 항상 검사한다 — 트리 모양, 패스가 복제한 스타일 수,
//!    프레임을 반복해도 egui 메모리가 늘지 않는지.
//! 2. 시간 예산은 `ELM_MAGIC_PERF=1`일 때만 검사한다.

#[path = "harness.rs"]
mod harness;

use elm_magic::prelude::*;
use harness::*;

/// 참조 트리 모양은 벤치 수치의 의미다 — 행 1개 = `Row` + `Text` + `Button` 3노드,
/// 여기에 루트 `Col` + 머리 행 3노드 + 꼬리 텍스트 1노드가 더해진다.
#[test]
fn reference_tree_shape_is_stable() {
    for n in [10usize, 100, 1_000] {
        let app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        });
        assert_eq!(
            node_count(app.element()),
            3 * n + 5,
            "행 {n}개의 트리 모양이 바뀌었다 — 벤치 수치의 기준이 흔들린다"
        );
    }
}

/// 어댑터가 한 패스에 복제하는 `ResolvedStyle`은 노드 수를 넘지 않는다
/// (노드당 1개 — 자식 루프/intrinsic은 `Pass`에 기록하지 않는다).
#[test]
fn pass_records_at_most_one_style_per_node() {
    let egui_ctx = egui::Context::default();
    for n in [10usize, 100, 1_000] {
        let mut app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        });
        let nodes = node_count(app.element());
        let tree = app.element().clone();
        let arena = &mut app.ctx.arena;
        let mut recorded = 0usize;
        let mut out = egui_ctx.run_ui(raw_input(), |ui| {
            let pass = elm_magic_egui::render(ui, &tree, arena);
            recorded = pass.styles.len();
        });
        out.textures_delta.clear();
        assert!(
            recorded <= nodes,
            "행 {n}개: 복제한 스타일 {recorded}개 > 노드 {nodes}개"
        );
    }
}

/// 프레임을 반복해도 egui 메모리(temp data)가 자라지 않는다.
///
/// 어댑터는 노드마다 `NodeMemory`를 temp data에 넣는다 — 키가 노드 순번이라
/// 같은 트리에서는 매 프레임 같은 키를 덮어쓴다. 트리 크기가 바뀌면 늘었다 줄어든다.
#[test]
fn egui_memory_is_stable_across_frames() {
    let egui_ctx = egui::Context::default();
    let n = 200;
    let mut app = mount_with::<BenchList>(BenchListProps {
        items: Some(items(n)),
        ..Default::default()
    });
    let tree = app.element().clone();
    let arena = &mut app.ctx.arena;
    let mut render = || {
        let mut out = egui_ctx.run_ui(raw_input(), |ui| {
            elm_magic_egui::render(ui, &tree, arena);
        });
        out.textures_delta.clear();
    };
    for _ in 0..10 {
        render();
    }
    let a = egui_ctx.data(|d| d.len());
    for _ in 0..10 {
        render();
    }
    let b = egui_ctx.data(|d| d.len());
    println!(
        "egui temp data 항목 수: {a} → {b} (노드 {}개)",
        node_count(&tree)
    );
    assert!(
        b <= a,
        "프레임 반복으로 egui 메모리가 늘었다: {a} → {b} (누수 후보)"
    );
}

/// 옵트인 시간 예산 — `ELM_MAGIC_PERF=1 cargo test ... --release`.
///
/// 기준은 **릴리스 모드 + 헤드리스**에서 어댑터 한 프레임이다(= 되그리기 1회).
#[test]
fn frame_budget_under_load() {
    if !is_release() {
        eprintln!("skip: debug 빌드");
        return;
    }
    if !perf_assert_enabled() {
        eprintln!("skip: ELM_MAGIC_PERF=1 이 아니다 (시간 단언은 옵트인)");
        return;
    }
    let egui_ctx = egui::Context::default();
    for (n, budget_ms) in [(100usize, 4.0f64), (1_000, 20.0), (3_000, 60.0)] {
        let mut app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        });
        let tree = app.element().clone();
        let arena = &mut app.ctx.arena;
        let iters = iters_for(n);
        let mut samples: Vec<f64> = Vec::new();
        for i in 0..(iters + 5) {
            let t = std::time::Instant::now();
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                elm_magic_egui::render(ui, &tree, arena);
            });
            out.textures_delta.clear();
            if i >= 5 {
                samples.push(t.elapsed().as_secs_f64() * 1e3);
            }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = samples[samples.len() / 2];
        println!("행 {n}개: median {median:.3} ms (예산 {budget_ms:.1} ms)");
        assert!(
            median <= budget_ms,
            "행 {n}개 프레임이 예산을 넘었다: {median:.3} ms > {budget_ms:.1} ms"
        );
    }
}
