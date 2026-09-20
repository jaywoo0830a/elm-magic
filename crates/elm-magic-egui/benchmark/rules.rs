//! H2 — 스타일 규칙 수(R) 스윕: `resolve_nodes`가 정말 `O(R)`인가.
//!
//! `resolve_nodes`는 **호출마다** 전역 등록부를 잠그고, 모든 규칙을 매칭해 `Vec`에
//! 모은 뒤 `specificity`로 정렬한다. 그러면 노드 하나의 해석 비용은 등록 규칙 수에
//! **선형**이어야 하고, 어댑터는 노드마다 이를 여러 번 부른다.
//!
//! 이 파일은 전역 레지스트리를 직접 비우고 채우므로 **단독 프로세스**여야 한다 —
//! `benchmark/harness.rs`의 `css!` 규칙과 섞이지 않도록 자기 파일로 분리했다.

#[path = "harness.rs"]
mod harness;

use elm_magic::style::{Edges, Node as StyleNode, Palette, State as StyleState, StyleSpec};
use harness::*;

/// `r`개 규칙을 등록한다 (`.bench_r0` … `.bench_r{r-1}`).
fn register_rules(r: usize) {
    if r == 0 {
        return;
    }
    let owned: Vec<(String, StyleSpec)> = (0..r)
        .map(|i| {
            (
                format!(".bench_r{i}"),
                StyleSpec {
                    gap: Some(1.0),
                    padding: Some(Edges::new(1.0, 1.0, 1.0, 1.0)),
                    ..Default::default()
                },
            )
        })
        .collect();
    let entries: Vec<(&str, StyleSpec)> =
        owned.iter().map(|(s, spec)| (s.as_str(), *spec)).collect();
    elm_magic::style::register(entries);
}

/// 규칙 `r`개일 때 해석 1회의 나노초. `matching`이면 **마지막 규칙에 맞는** 노드를 쓴다.
fn ns_per_call(r: usize, matching: bool) -> f64 {
    let palette = Palette::dark();
    let own = ["bench_miss".to_string()];
    let hit = [format!("bench_r{}", r.saturating_sub(1))];
    let classes: &[String] = if matching { &hit } else { &own };
    let node = StyleNode::new("row", classes);
    let path = [node];
    let inner = (2_000_000 / (r + 1)).clamp(200, 20_000);

    let mut samples: Vec<f64> = Vec::new();
    for _ in 0..3 {
        let t = std::time::Instant::now();
        let mut acc = 0usize;
        for _ in 0..inner {
            let s = elm_magic::style::resolve_nodes(&path, StyleState::NONE, None, &palette);
            acc += s.is_empty() as usize;
        }
        std::hint::black_box(acc);
        samples.push(t.elapsed().as_secs_f64() * 1e9 / inner as f64);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    samples[samples.len() / 2]
}

#[test]
fn h2_rule_count_sweep() {
    if !is_release() {
        eprintln!("⚠ debug 빌드다 — `--release`로 돌려야 수치가 의미 있다.");
    }
    println!("| 규칙 수 | 비매칭 ns/회 | 매칭 ns/회 | 비매칭/규칙 (ns) |");
    println!("|---|---|---|---|");
    for r in [0usize, 10, 50, 200, 1_000] {
        elm_magic::style::reset_for_tests();
        register_rules(r);
        let miss = ns_per_call(r, false);
        let hit = ns_per_call(r, true);
        println!(
            "| {r} | {miss:.1} | {hit:.1} | {:.2} |",
            if r == 0 { 0.0 } else { miss / r as f64 }
        );
        println!("  (등록 확인: style::len() = {})", elm_magic::style::len());
    }
    // 마지막 스윕(1,000 규칙)에서 노드 하나 해석이 w µs라면, 노드 수 × 호출 배수가 곱해진다.
    let measured = ns_per_call(1_000, false);
    println!(
        "\n1000 규칙에서 1회 = {:.2} µs → 3,000노드 × 3회 해석이면 약 {:.1} ms",
        measured / 1_000.0,
        measured * 3.0 * 3_000.0 / 1e6
    );
}
