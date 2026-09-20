//! 병목 후보를 하나씩 분리해 잰다 — 최적화 우선순위를 정하기 위한 표.
//!
//! | # | 가설 | 재는 방법 |
//! |---|---|---|
//! | H1 | `resolve_nodes`가 **호출마다** 규칙 수만큼 선형 스캔 + `Vec` + `sort` + `Mutex` | 같은 노드를 2만 번 해석 |
//! | H3 | intrinsic 예산 계산이 자식 트리를 **다시** 돌아 깊이에 초선형 | 깊이 D 중첩 vs 같은 노드 수의 평면 |
//! | H4 | `key={id}`가 매 프레임 `format!("{:?}")` + `HashMap` 삽입 | keyed vs 위치 기반 같은 트리 |
//! | H5 | 클래스 매칭이 붙는 순간 노드당 비용이 뜀 | 클래스 없는 트리 vs 같은 모양 |
//! | H6 | 노드마다 egui temp data 왕복(`get_temp` + `insert_temp`) | 같은 횟수의 왕복을 직접 |
//! | H7 | `Pass.styles`가 스타일 있는 노드마다 `ResolvedStyle`을 **복제**해 쌓는다 | 스타일 수 = 노드 수인 트리 |

#[path = "harness.rs"]
mod harness;

use elm_magic::prelude::*;
use elm_magic::style::{Node as StyleNode, Palette, State as StyleState};
use harness::*;

// 클래스 없는 나무 — 스타일 매칭이 0회일 때의 바닥.
elm_magic::view! {
    fn PlainList(n = 0) {
        <Col>
            <For each={0..n} as={i}>
                <Row>
                    <Text>"{i}"</Text>
                </Row>
            </For>
        </Col>
    }
}

// 같은 모양 + 클래스 — 매칭/적용이 일어날 때의 비용.
elm_magic::view! {
    fn ClassedList(n = 0) {
        <Col class="bench_root">
            <For each={0..n} as={i}>
                <Row class="bench_row">
                    <Text class="bench_row_text">"{i}"</Text>
                </Row>
            </For>
        </Col>
    }
}

// 깊이 `depth`로 중첩한 컨테이너 — intrinsic 재순회(O(깊이²))를 드러낸다.
elm_magic::view! {
    fn DeepCol(depth = 1) {
        <Col class="bench_root">
            <For each={1..depth} as={d}>
                <Col class="bench_root">
                    <Text>"{d}"</Text>
                </Col>
            </For>
        </Col>
    }
}

fn render_once<C: Component + 'static>(
    egui_ctx: &egui::Context,
    app: &mut elm_magic::testing::TestApp<C>,
) {
    let tree = app.element().clone();
    let arena = &mut app.ctx.arena;
    let mut out = egui_ctx.run_ui(raw_input(), |ui| {
        elm_magic_egui::render(ui, &tree, arena);
    });
    out.textures_delta.clear();
}

fn frame_once<C: Component>(props: &C::Props) -> usize {
    let mut ctx = elm_magic::Ctx::new();
    let tree = elm_magic::frame::<C>(&mut ctx, props);
    node_count(&tree)
}

/// H1 — 스타일 해석 **한 번**의 비용 (ns/회) 과, 경로/상태가 늘 때의 증가.
fn ns_per_call(label: &str, warmup: usize, iters: usize, mut f: impl FnMut()) {
    for _ in 0..warmup {
        f();
    }
    let mut samples: Vec<f64> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = std::time::Instant::now();
        f();
        samples.push(t.elapsed().as_secs_f64() * 1e9);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    println!("| {label} | {:.1} ns |", samples[samples.len() / 2]);
}

#[test]
fn h1_resolve_call_cost() {
    println!(
        "\n### H1 스타일 해석 1회 (등록 규칙 {}개)\n",
        elm_magic::style::len()
    );
    println!("| 해석 | ns/회 |");
    println!("|---|---|");
    let palette = Palette::dark();
    let classes = ["bench_row".to_string()];
    let ancestor_classes = ["bench_root".to_string()];
    let self_node = StyleNode::new("row", &classes);
    let anc = StyleNode::new("col", &ancestor_classes);
    let path = [self_node, anc];

    ns_per_call("조상 0 (자기 자신만)", 1000, 20_000, || {
        let s = elm_magic::style::resolve_nodes(&path[..1], StyleState::NONE, None, &palette);
        std::hint::black_box(s.is_empty());
    });
    ns_per_call("조상 1 (후손 셀렉터 가능)", 1000, 20_000, || {
        let s = elm_magic::style::resolve_nodes(&path, StyleState::NONE, None, &palette);
        std::hint::black_box(s.is_empty());
    });
    ns_per_call("조상 1 + :hover", 1000, 20_000, || {
        let s = elm_magic::style::resolve_nodes(
            &path,
            StyleState::new(true, false, false, false),
            None,
            &palette,
        );
        std::hint::black_box(s.is_empty());
    });
    println!("\n(규칙 수 스윕은 `benchmark_rules`가 맡는다 — 규칙당 약 5ns로 **선형**이다)");
}
/// H3 — 깊이에 따른 초선형 증가 (intrinsic 예산 계산이 자식 트리를 다시 돈다).
#[test]
fn h3_intrinsic_rewalk() {
    for depth in [20usize, 100, 400] {
        let iters = iters_for(depth * 2);
        let egui_ctx = egui::Context::default();
        let mut deep = mount_with::<DeepCol>(DeepColProps {
            depth: Some(depth as i32),
            ..Default::default()
        });
        let mut flat = mount_with::<ClassedList>(ClassedListProps {
            n: Some(depth as i32),
            ..Default::default()
        });
        let deep_nodes = node_count(deep.element());
        let flat_nodes = node_count(flat.element());
        println!("\n### H3 깊이 {depth} — 중첩 {deep_nodes}노드 vs 평면 {flat_nodes}노드\n");
        Table::header();
        let mut t = Table::new();
        t.measure("H3 중첩 Col (깊이 D)", deep_nodes, 3, iters, || {
            render_once(&egui_ctx, &mut deep);
        });
        t.measure("H3 평면 Col (N=D)", flat_nodes, 3, iters, || {
            render_once(&egui_ctx, &mut flat);
        });
        if let (Some(a), Some(b)) = (
            t.get("H3 중첩 Col (깊이 D)", deep_nodes),
            t.get("H3 평면 Col (N=D)", flat_nodes),
        ) {
            let per_deep = a.median / deep_nodes as f64;
            let per_flat = b.median / flat_nodes as f64;
            println!(
                "- 노드당: 중첩 {per_deep:.4} ms/노드, 평면 {per_flat:.4} ms/노드 → {:.2}x",
                per_deep / per_flat.max(f64::MIN_POSITIVE)
            );
        }
    }
}

/// H4 — keyed 슬롯(`key={t.id}`) vs 위치 기반 슬롯.
#[test]
fn h4_keyed_slots() {
    for n in sizes() {
        let iters = iters_for(n);
        let props_k = BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        };
        let props_p = BenchListPositionalProps {
            items: Some(items(n)),
            ..Default::default()
        };
        println!("\n### H4 keyed vs 위치 기반 — 행 {n}개\n");
        Table::header();
        let mut t = Table::new();
        t.measure("H4 트리 재생성 (keyed)", n, 5, iters, || {
            std::hint::black_box(frame_once::<BenchList>(&props_k));
        });
        t.measure("H4 트리 재생성 (위치)", n, 5, iters, || {
            std::hint::black_box(frame_once::<BenchListPositional>(&props_p));
        });
        if let (Some(a), Some(b)) = (
            t.get("H4 트리 재생성 (keyed)", n),
            t.get("H4 트리 재생성 (위치)", n),
        ) {
            println!(
                "- keyed 슬롯이 더한 비용 = {:.3} ms ({:.1}%)",
                a.median - b.median,
                (a.median / b.median - 1.0) * 100.0
            );
        }
    }
}

/// H5 — 클래스 매칭 유무: 같은 모양에서 클래스만 뺀 트리와 비교.
#[test]
fn h5_class_matching() {
    let egui_ctx = egui::Context::default();
    for n in sizes() {
        let iters = iters_for(n);
        let mut plain = mount_with::<PlainList>(PlainListProps {
            n: Some(n as i32),
            ..Default::default()
        });
        let mut classed = mount_with::<ClassedList>(ClassedListProps {
            n: Some(n as i32),
            ..Default::default()
        });
        println!("\n### H5 클래스 유무 — 행 {n}개\n");
        Table::header();
        let mut t = Table::new();
        t.measure("H5 클래스 없음", n, 5, iters, || {
            render_once(&egui_ctx, &mut plain);
        });
        t.measure("H5 클래스 있음", n, 5, iters, || {
            render_once(&egui_ctx, &mut classed);
        });
        if let (Some(a), Some(b)) = (t.get("H5 클래스 없음", n), t.get("H5 클래스 있음", n))
        {
            println!(
                "- 클래스 매칭이 더한 비용 = {:.3} ms ({:.1}%)",
                b.median - a.median,
                (b.median / a.median - 1.0) * 100.0
            );
        }
    }
}

/// H6 — 노드마다 하는 egui temp data 왕복 (어댑터의 `read_state`/`write_state`와 같은 횟수).
#[test]
fn h6_egui_temp_roundtrip() {
    #[derive(Clone, Copy)]
    struct Probe {
        rect: egui::Rect,
        focused: bool,
    }
    let egui_ctx = egui::Context::default();
    Table::header();
    let mut t = Table::new();
    for n in [1_000usize, 3_000] {
        t.measure("H6 노드당 temp 왕복", n, 3, 50, || {
            for i in 0..n {
                let id = egui::Id::new(("bench-mem", i));
                let got = egui_ctx.memory(|m| m.data.get_temp::<Probe>(id));
                std::hint::black_box(got.map_or(0.0, |p| p.rect.width() + f32::from(p.focused)));
                let hovered = egui_ctx.pointer_hover_pos();
                std::hint::black_box(hovered);
                egui_ctx.memory_mut(|m| {
                    m.data.insert_temp(
                        id,
                        Probe {
                            rect: egui::Rect::ZERO,
                            focused: false,
                        },
                    )
                });
            }
        });
    }
}

/// H8 — **egui 조작 1회의 단가**. 어댑터가 노드마다 만드는 Ui 조작의 비용을 직접 잰다.
///
/// 어댑터는 노드 하나를 그릴 때 여러 번의 `Ui` 조작을 만든다 — 스코프, `Frame`,
/// 레이아웃 전환, 위젯 추가, 크기 예약, 메모리 왕복. 그 단가를 알아야
/// "어댑터 오버헤드 2.3ms"가 **어디서** 나오는지 계산으로 귀속할 수 있다.
fn per_op(label: &str, n: usize, mut f: impl FnMut(&mut egui::Ui)) {
    let egui_ctx = egui::Context::default();
    let sample = |f: &mut dyn FnMut(&mut egui::Ui)| -> f64 {
        let t = std::time::Instant::now();
        let mut out = egui_ctx.run_ui(raw_input(), |ui| {
            for _ in 0..n {
                f(ui);
            }
        });
        out.textures_delta.clear();
        t.elapsed().as_secs_f64() * 1e9 / n as f64
    };
    for _ in 0..3 {
        sample(&mut f);
    }
    let mut samples = [0.0f64; 5];
    for s in samples.iter_mut() {
        *s = sample(&mut f);
    }
    let empty = {
        let mut f = |_ui: &mut egui::Ui| {};
        let mut best = f64::MAX;
        for _ in 0..3 {
            best = best.min(sample(&mut f));
        }
        best
    };
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let per = samples[2];
    println!("| {label} | {n} | {:.0} |", per);
    println!(
        "| ↑ 빈 패스 차감 | — | {:.0} |",
        (per - empty / n as f64).max(0.0)
    );
}

#[test]
fn h8_egui_op_unit_cost() {
    if !is_release() {
        eprintln!("⚠ debug 빌드다 — `--release`로 돌려야 수치가 의미 있다.");
    }
    let n = 1_000;
    println!("\n### H8 egui 조작 단가 (한 패스 안에서 {n}회, ns/회)\n");
    println!("| egui 조작 | 회 | ns/회 |");
    println!("|---|---|---|");

    per_op("빈 패스 (기준)", n, |_ui| {});
    per_op("ui.scope(|_| {})", n, |ui| {
        ui.scope(|_| {});
    });
    per_op("ui.scope(라벨 1개)", n, |ui| {
        ui.scope(|ui| {
            ui.label("item 42");
        });
    });
    per_op("Frame::NONE.show", n, |ui| {
        egui::Frame::NONE.show(ui, |_ui| {});
    });
    per_op("Frame inner_margin.show", n, |ui| {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(4, 2))
            .show(ui, |_ui| {});
    });
    per_op("with_layout(가로)", n, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |_ui| {});
    });
    per_op("ui.add(Label)", n, |ui| {
        ui.add(egui::Label::new("item 42"));
    });
    per_op("ui.add(Button)", n, |ui| {
        ui.add(egui::Button::new("x"));
    });
    per_op("allocate_exact_size", n, |ui| {
        ui.allocate_exact_size(egui::vec2(80.0, 18.0), egui::Sense::hover());
    });
    per_op("set_height + item_spacing", n, |ui| {
        ui.set_height(22.0);
        ui.spacing_mut().item_spacing.x = 8.0;
    });
    per_op("Id::new(튜플)", n, |ui| {
        std::hint::black_box(egui::Id::new(("elm-magic-style", ui.next_auto_id())));
    });
}

/// H7 — `Pass.styles`가 스타일 있는 노드마다 `ResolvedStyle`을 복제해 쌓는 비용.
#[test]
fn h7_pass_style_clones() {
    let egui_ctx = egui::Context::default();
    Table::header();
    let mut t = Table::new();
    for n in sizes() {
        let iters = iters_for(n);
        let mut app = mount_with::<BenchList>(BenchListProps {
            items: Some(items(n)),
            ..Default::default()
        });
        let tree = app.element().clone();
        let arena = &mut app.ctx.arena;
        let mut last = 0usize;
        t.measure("H7 어댑터 렌더 + styles 수집", n, 5, iters, || {
            let mut out = egui_ctx.run_ui(raw_input(), |ui| {
                let pass = elm_magic_egui::render(ui, &tree, arena);
                last = pass.styles.len();
            });
            out.textures_delta.clear();
        });
        println!("- 마지막 패스가 복제한 `ResolvedStyle` 수 = {last}개 (행 {n}개)");
    }
}
