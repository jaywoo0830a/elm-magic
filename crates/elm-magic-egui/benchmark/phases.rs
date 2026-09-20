//! 단계 분해 — elm-magic 한 프레임을 층별로 나눠 잰다.
//!
//! | 단계 | 재는 것 |
//! |---|---|
//! | P0 | egui 바닥 — 위젯 없는 한 패스(패스 고정비) |
//! | P1 | 상태 → `Element` 트리 재생성 (`elm_magic::frame`) |
//! | P2 | 스타일 해석만 — 노드마다 1회 (`resolved_style_in`) |
//! | P3 | 어댑터 전체 (`elm_magic_egui::render` + egui 레이아웃/페인트리스트) |
//! | P5 | 테셀레이션 (`Context::tessellate`, GPU 이전 CPU 비용) |
//!
//! P3에서 P0을 빼면 "어댑터가 egui 위에 얹은 비용",
//! 다시 P1+P2를 빼면 "어댑터가 같은 노드를 여러 번 해석하며 쓴 비용"이 남는다.

#[path = "harness.rs"]
mod harness;

use elm_magic::prelude::*;
use harness::*;

#[test]
fn phase_breakdown() {
    if !is_release() {
        eprintln!("⚠ debug 빌드다 — `--release`로 돌려야 수치가 의미 있다.");
    }
    for n in sizes() {
        let iters = iters_for(n);
        let mut app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        });
        let tree = app.element().clone();
        let nodes = node_count(&tree);
        let egui_ctx = egui::Context::default();
        let palette = elm_magic::style::Palette::dark();

        println!("\n### N={n} (행 {n}개) — 트리 노드 {nodes}개\n");
        Table::header();
        let mut t = Table::new();

        // P0 — egui 패스 고정비 (위젯 없음)
        t.measure("P0 egui 빈 패스", n, 5, iters, || {
            let mut out = egui_ctx.run_ui(raw_input(), |_ui| {});
            out.textures_delta.clear();
        });

        // P1 — 트리 재생성 (props 복제 + 컴포넌트 본문 + keyed 슬롯)
        {
            let mut ctx = elm_magic::Ctx::new();
            let props = BenchListProps {
                items: Some(items(n)),
                ..Default::default()
            };
            t.measure("P1 트리 재생성", n, 5, iters, || {
                let tree = elm_magic::frame::<BenchList>(&mut ctx, &props);
                std::hint::black_box(&tree);
            });
        }

        // P2 — 스타일 해석 1회/노드 (어댑터가 하는 일의 최소치)
        {
            let mut ancestors: Vec<&elm_magic::Element> = Vec::new();
            let mut inherited = Vec::new();
            t.measure("P2 스타일 해석 1회/노드", n, 5, iters, || {
                let nodes = resolve_walk(&tree, &mut ancestors, &mut inherited, &palette);
                std::hint::black_box(nodes);
            });
        }

        // P3 — 어댑터 전체 (`render`: 스타일 기록 포함 = 테스트 경로)
        {
            let arena = &mut app.ctx.arena;
            t.measure("P3 어댑터 렌더(+egui)", n, 5, iters, || {
                let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                    let pass = elm_magic_egui::render(ui, &tree, arena);
                    std::hint::black_box(pass.styles.len());
                });
                out.textures_delta.clear();
            });
        }

        // P4 — 앱 경로 (`render_fast`: 스타일 기록 없음 + 해석 캐시)
        {
            let arena = &mut app.ctx.arena;
            t.measure("P4 어댑터 (render_fast)", n, 5, iters, || {
                let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                    elm_magic_egui::render_fast(ui, &tree, arena);
                });
                out.textures_delta.clear();
            });
        }

        // P5 — 테셀레이션 (shapes는 P3와 같은 트리에서 한 번 뜬 것)
        {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                elm_magic_egui::render(ui, &tree, &mut app.ctx.arena);
            });
            let shapes = std::mem::take(&mut out.shapes);
            out.textures_delta.clear();
            let ppp = out.pixels_per_point;
            t.measure("P5 테셀레이션", n, 5, iters, || {
                let prims = egui_ctx.tessellate(shapes.clone(), ppp);
                std::hint::black_box(prims.len());
            });
        }

        // 파생 비율 — 어디가 지배하는가
        if let (Some(p0), Some(p1), Some(p2), Some(p3), Some(p4)) = (
            t.get("P0 egui 빈 패스", n),
            t.get("P1 트리 재생성", n),
            t.get("P2 스타일 해석 1회/노드", n),
            t.get("P3 어댑터 렌더(+egui)", n),
            t.get("P4 어댑터 (render_fast)", n),
        ) {
            let elm_side = p1.median + p2.median;
            println!(
                "\n- P3/P0 = {:.1}x, P3/(P1+P2) = {:.2}x, 60fps 여유 = {:.0} fps",
                p3.ratio(p0),
                p3.median / elm_side.max(f64::MIN_POSITIVE),
                p3.fps()
            );
            println!(
                "- render_fast가 P3 대비 {:.2}x (= {:.3} ms 절약), P4 여유 = {:.0} fps",
                p4.median / p3.median.max(f64::MIN_POSITIVE),
                p3.median - p4.median,
                p4.fps()
            );
        }
    }
}
