//! 손으로 쓴 egui 기준선 — 같은 화면을 egui API로 직접 그린 것과 어댑터를 비교한다.
//!
//! | 기준선 | 무엇을 가정하는가 |
//! |---|---|
//! | B1 사람이 그대로 쓴 egui | `ui.horizontal` + `format!` 라벨 + `ui.button` — 매 프레임 문자열 생성 |
//! | B2 손 튜닝한 egui | 라벨을 **미리** 만들어 두고 `&str`로 넘긴다 (할당 0) |
//! | B3 elm-magic 어댑터 | 매 프레임 트리 재생성 + 스타일 해석 + 위젯 |
//!
//! 세 기준선 모두 **같은 egui 패스 고정비**(P0)를 포함한다. 그래서 비율 차이는
//! "위에 얹은 층"의 비용이다. 페인트 명령 수(`shapes`)도 함께 센다 — 어댑터는
//! egui 기본 스타일을 리셋하므로 기본 비주얼보다 그리는 것이 적을 수 있다.

#[path = "harness.rs"]
mod harness;

use elm_magic::prelude::*;
use harness::*;

/// B1 — 사람이 그대로 쓴 egui (매 프레임 `format!`).
fn baseline_naive(ui: &mut egui::Ui, items: &[Item], _selected: usize) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(format!("items: {}", items.len()));
            if ui.button("inc").clicked() {}
        });
        for it in items {
            ui.horizontal(|ui| {
                ui.label(it.label.clone());
                if ui.button("x").clicked() {}
            });
        }
        ui.label(format!("selected: {_selected}"));
    });
}

/// B2 — 손 튜닝: 라벨은 미리 만들어 `&str`로만 넘긴다.
fn baseline_tuned(ui: &mut egui::Ui, head: &str, labels: &[String], tail: &str) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(head);
            if ui.button("inc").clicked() {}
        });
        for label in labels {
            ui.horizontal(|ui| {
                ui.label(label.as_str());
                if ui.button("x").clicked() {}
            });
        }
        ui.label(tail);
    });
}

#[test]
fn adapter_vs_hand_written_egui() {
    if !is_release() {
        eprintln!("⚠ debug 빌드다 — `--release`로 돌려야 수치가 의미 있다.");
    }
    for n in sizes() {
        let iters = iters_for(n);
        let data = items(n);
        let head = format!("items: {n}");
        let labels: Vec<String> = data.iter().map(|i| i.label.clone()).collect();
        let tail = "selected: 0".to_string();

        let mut app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        });
        let tree = app.element().clone();
        let egui_ctx = egui::Context::default();

        println!("\n### N={n} — 손 egui vs 어댑터\n");
        Table::header();
        let mut t = Table::new();

        t.measure("B1 손 egui (naive)", n, 5, iters, || {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                baseline_naive(ui, &data, 0);
            });
            out.textures_delta.clear();
        });

        t.measure("B2 손 egui (tuned)", n, 5, iters, || {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                baseline_tuned(ui, &head, &labels, &tail);
            });
            out.textures_delta.clear();
        });

        let arena = &mut app.ctx.arena;
        t.measure("B3 elm-magic (render_fast)", n, 5, iters, || {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                elm_magic_egui::render_fast(ui, &tree, arena);
            });
            out.textures_delta.clear();
        });

        t.measure("B4 elm-magic (render + 기록)", n, 5, iters, || {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                let pass = elm_magic_egui::render(ui, &tree, arena);
                std::hint::black_box(pass.styles.len());
            });
            out.textures_delta.clear();
        });

        // 페인트 명령 수 — 그리는 양(GPU 쪽) 비교
        let shapes_of = |f: &mut dyn FnMut(&mut egui::Ui)| {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| f(ui));
            let n = out.shapes.len();
            let hist = shape_histogram(&out);
            out.textures_delta.clear();
            (n, hist)
        };
        let (s1, h1) = shapes_of(&mut |ui| baseline_naive(ui, &data, 0));
        let (s2, _) = shapes_of(&mut |ui| baseline_tuned(ui, &head, &labels, &tail));
        let (s3, h3) = shapes_of(&mut |ui| {
            elm_magic_egui::render_fast(ui, &tree, arena);
        });

        println!("\n| 페인트 명령 수 | 값 |");
        println!("|---|---|");
        println!("| B1 손 egui (naive) | {s1} |");
        println!("| B2 손 egui (tuned) | {s2} |");
        println!("| B3 elm-magic (render_fast) | {s3} |");
        println!("\n명령 구성 — B1 {h1:?}\n명령 구성 — B3 {h3:?}");

        if let (Some(b1), Some(b2), Some(b3), Some(b4)) = (
            t.get("B1 손 egui (naive)", n),
            t.get("B2 손 egui (tuned)", n),
            t.get("B3 elm-magic (render_fast)", n),
            t.get("B4 elm-magic (render + 기록)", n),
        ) {
            println!(
                "\n- 어댑터/B1 = {:.2}x, 어댑터/B2 = {:.2}x, B1/B2 = {:.2}x",
                b3.ratio(b1),
                b3.ratio(b2),
                b1.ratio(b2)
            );
            println!(
                "- 기록(Pass)을 켜면 {:.2}x 느려진다 (+{:.3} ms)",
                b4.median / b3.median.max(f64::MIN_POSITIVE),
                b4.median - b3.median
            );
        }
    }
}
