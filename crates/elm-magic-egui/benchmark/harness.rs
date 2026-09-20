//! 벤치마크 공용 하네스 — 헤드리스 egui 0.36.2 + elm-magic.
//!
//! `tests/`가 아니라 `benchmark/`에 두는 이유: 이 파일들은 **계약**이 아니라
//! **측정**이라 CI의 테스트 스위트(계약 290개)에 섞이지 않는다. Cargo의 자동 탐색은
//! `tests/` 디렉터리만 보므로, 실행하려면 `Cargo.toml`의 `[[test]]`로 **명시 등록**한다.
//!
//! ```text
//! cargo test -p elm-magic-egui --test benchmark_phases    --release -- --nocapture
//! cargo test -p elm-magic-egui --test benchmark_baselines --release -- --nocapture
//! cargo test -p elm-magic-egui --test benchmark_hotspots  --release -- --nocapture
//! cargo test -p elm-magic-egui --test benchmark_rules     --release -- --nocapture
//! cargo test -p elm-magic-egui --test benchmark_guard     --release -- --nocapture
//! ```
//!
//! ## 측정 규약
//!
//! - **릴리스 모드 전용**: debug 빌드는 10~50배 느려 의미가 없다. 각 측정은
//!   `cfg!(debug_assertions)`면 `SKIP`을 찍고 단언하지 않는다.
//! - **헤드리스**: `egui::Context::run_ui`로 창 없이 돈다(`tests/adapter.rs`와 같은
//!   하네스). `run_ui`는 뷰포트 전체를 덮는 루트 `Ui`를 주고, 폰트 아틀라스/갈레이
//!   캐시는 워밍업에서 채운다.
//! - **벽시계**: `Instant`로 잰다. 최소/중위/p90만 보고한다(최소 = 노이즈 바닥,
//!   중위 = 대표값, p90 = 꼬리). CI에서 단언하려면 `ELM_MAGIC_PERF=1`을 켠다.
//! - **단계 분리**: 트리 재생성 / 스타일 해석 / 어댑터 렌더 / 손 egui 기준선을
//!   따로 잰다 — 합계만 보면 어느 층이 지배하는지 알 수 없다.

#![allow(dead_code)]

use std::time::Instant;

use elm_magic::Widget;

/// 벤치 창 크기 — 일반적인 데스크톱 창(1280×800)을 가정한다.
pub const W: f32 = 1280.0;
pub const H: f32 = 800.0;

// ── 컴포넌트 (측정 대상) ──────────────────────────────────────

/// 목록 한 행의 데이터 — 실제 앱처럼 `String` 라벨을 가진다.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Item {
    pub id: u64,
    pub label: String,
}

/// `n`개의 항목. 라벨은 **미리** 만든다(트리는 매 프레임 복제만 한다).
pub fn items(n: usize) -> Vec<Item> {
    (0..n)
        .map(|i| Item {
            id: i as u64,
            label: format!("item {i}"),
        })
        .collect()
}

// 실제 앱과 비슷한 규칙 수(6개) — 병목의 `O(규칙수)` 항을 드러낸다.
// `benchmark_rules.rs`가 규칙 수만 바꿔 같은 해석 비용을 다시 잰다.
elm_magic::css! {
    .bench_root      { padding: 8; gap: 2; }
    .bench_head      { height: 24; gap: 8; }
    .bench_head_text { flex-grow: 1; }
    .bench_row       { height: 22; gap: 8; padding: 2 4; }
    .bench_row_text  { flex-grow: 1; color: text; }
    .bench_btn       { padding: 2 8; }
}

// 측정용 목록 컴포넌트 — 컨테이너/텍스트/버튼/클래스/`<For>`가 모두 들어간다.
elm_magic::view! {
    pub fn BenchList(items: Vec<Item> = vec![], selected = 0) {
        <Col class="bench_root">
            <Row class="bench_head">
                <Text class="bench_head_text">"items: {items.len()}"</Text>
                <Button class="bench_btn" on_click={selected += 1}>"inc"</Button>
            </Row>
            <For each={items.clone()} as={t} key={t.id}>
                <Row class="bench_row">
                    <Text class="bench_row_text">{t.label.clone()}</Text>
                    <Button class="bench_btn">"x"</Button>
                </Row>
            </For>
            "selected: {selected}"
        </Col>
    }
}

// 같은 모양이지만 **위치 기반** 슬롯(`key` 없음) — keyed 슬롯 비용과 비교한다.
elm_magic::view! {
    pub fn BenchListPositional(items: Vec<Item> = vec![], selected = 0) {
        <Col class="bench_root">
            <Row class="bench_head">
                <Text class="bench_head_text">"items: {items.len()}"</Text>
                <Button class="bench_btn" on_click={selected += 1}>"inc"</Button>
            </Row>
            <For each={items.clone()} as={t}>
                <Row class="bench_row">
                    <Text class="bench_row_text">{t.label.clone()}</Text>
                    <Button class="bench_btn">"x"</Button>
                </Row>
            </For>
            "selected: {selected}"
        </Col>
    }
} // ── 트리 순회 (스타일 해석 단계 분리용) ────────────────────────

/// 트리 전체 노드 수 (컨테이너면 자식 재귀, 리프면 1).
pub fn node_count(el: &elm_magic::Element) -> usize {
    1 + el
        .children()
        .map(|c| c.iter().map(node_count).sum())
        .unwrap_or(0)
}

/// 어댑터와 **같은 규칙**으로 노드마다 스타일을 한 번씩 해석한다.
///
/// 어댑터는 여기에 더해 intrinsic 예산 계산(`intrinsic_main`)과 자식 루프에서
/// 같은 노드를 여러 번 해석한다 — 그 차이가 `phases.rs`의 P2 vs P3 비교다.
pub fn resolve_walk<'a>(
    el: &'a elm_magic::Element,
    ancestors: &mut Vec<&'a elm_magic::Element>,
    inherited: &mut Vec<elm_magic::style::ResolvedStyle>,
    palette: &elm_magic::style::Palette,
) -> usize {
    let style = el.resolved_style_in(
        ancestors,
        elm_magic::style::State::new(false, false, false, el.is_disabled()),
        inherited.last(),
        palette,
    );
    std::hint::black_box(&style);
    ancestors.insert(0, el);
    inherited.push(style);
    let mut nodes = 1;
    if let Some(children) = el.children() {
        for c in children {
            nodes += resolve_walk(c, ancestors, inherited, palette);
        }
    }
    inherited.pop();
    ancestors.remove(0);
    nodes
}

// ── 시간 측정 ────────────────────────────────────────────────

/// 한 측정의 통계 (밀리초).
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub min: f64,
    pub median: f64,
    pub p90: f64,
}

impl Stats {
    /// 이 측정이 기준선보다 몇 배인가 (1.0 = 같음).
    pub fn ratio(&self, base: &Stats) -> f64 {
        if base.median > 0.0 {
            self.median / base.median
        } else {
            f64::INFINITY
        }
    }

    /// 초당 프레임 수 (중위값 기준) — 16.6ms 예산 판정용.
    pub fn fps(&self) -> f64 {
        if self.median > 0.0 {
            1000.0 / self.median
        } else {
            f64::INFINITY
        }
    }
}

/// 측정 결과를 담는 표 — `--nocapture`로 돌리면 마크다운 행으로 찍힌다.
#[derive(Default)]
pub struct Table {
    rows: Vec<(String, usize, Stats)>,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    /// 워밍업 `warmup`회는 버리고 `iters`회를 재서 `phase`/`n` 이름으로 담는다.
    ///
    /// 릴리스가 아니면 `f`를 한 번만 불러 동작만 확인하고 `SKIP`을 찍는다.
    pub fn measure(
        &mut self,
        phase: &str,
        n: usize,
        warmup: usize,
        iters: usize,
        mut f: impl FnMut(),
    ) {
        if cfg!(debug_assertions) {
            f();
            println!("| {phase} | {n} | SKIP (debug) | — | — |");
            return;
        }
        for _ in 0..warmup {
            f();
        }
        let iters = iters.max(1);
        let mut samples: Vec<f64> = Vec::with_capacity(iters);
        for _ in 0..iters {
            let t = Instant::now();
            f();
            samples.push(t.elapsed().as_secs_f64() * 1e3);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let stats = Stats {
            min: samples[0],
            median: samples[samples.len() / 2],
            p90: samples[(samples.len() * 9 / 10).min(samples.len() - 1)],
        };
        println!(
            "| {phase} | {n} | {:.3} | {:.3} | {:.3} |",
            stats.min, stats.median, stats.p90
        );
        self.rows.push((phase.to_string(), n, stats));
    }

    pub fn get(&self, phase: &str, n: usize) -> Option<&Stats> {
        self.rows
            .iter()
            .find(|(p, m, _)| p == phase && *m == n)
            .map(|(_, _, s)| s)
    }

    /// 표 머리 — 측정 시작 전에 한 번 찍는다.
    pub fn header() {
        println!("| 단계 | 노드수 | min (ms) | median (ms) | p90 (ms) |");
        println!("|---|---|---|---|---|");
    }
}

/// 페인트 명령의 **종류별** 개수 (그리는 양의 구성 — 어댑터가 무엇을 더 그리는가).
pub fn shape_histogram(out: &egui::FullOutput) -> Vec<(String, usize)> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for clipped in &out.shapes {
        let key = match &clipped.shape {
            egui::Shape::Noop => "Noop",
            egui::Shape::Vec(_) => "Vec",
            egui::Shape::Circle(_) => "Circle",
            egui::Shape::Ellipse(_) => "Ellipse",
            egui::Shape::LineSegment { .. } => "LineSegment",
            egui::Shape::Path(_) => "Path",
            egui::Shape::Rect(_) => "Rect",
            egui::Shape::Text(_) => "Text",
            egui::Shape::Mesh(_) => "Mesh",
            egui::Shape::Callback(_) => "Callback",
            _ => "기타",
        };
        *map.entry(key.to_string()).or_default() += 1;
    }
    map.into_iter().collect()
}

// ── 실행 파라미터 ────────────────────────────────────────────

/// 반복 횟수 — 노드 수에 따라 시간이 선형 이상으로 늘어 작은 트리는 촘촘히 잰다.
pub fn iters_for(n: usize) -> usize {
    match n {
        0..=200 => 100,
        201..=1_000 => 30,
        _ => 8,
    }
}

/// 벤치 노드 수 목록 — 환경변수로 덮어쓴다(`ELM_MAGIC_BENCH_N=100,1000`).
pub fn sizes() -> Vec<usize> {
    match std::env::var("ELM_MAGIC_BENCH_N") {
        Ok(v) => v
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .collect(),
        Err(_) => vec![100, 1_000, 3_000],
    }
}

/// 릴리스 모드인가 (측정이 의미 있는가).
pub fn is_release() -> bool {
    !cfg!(debug_assertions)
}

/// `ELM_MAGIC_PERF=1`이면 시간 단언을 켠다 (CI 기본은 꺼짐 — 벽시계는 flaky).
pub fn perf_assert_enabled() -> bool {
    std::env::var("ELM_MAGIC_PERF").is_ok_and(|v| v != "0")
}

/// 헤드리스 입력 (텍스트 아틀라스는 첫 프레임에 만들어진다).
pub fn raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(W, H),
        )),
        ..Default::default()
    }
}
