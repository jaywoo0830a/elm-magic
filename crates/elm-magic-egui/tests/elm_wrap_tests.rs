//! elm-magic 최소 재현 — **freedf-gui 코드를 쓰지 않는다**(elm-magic + egui만).
//!
//! `src/style.rs`/`ui/*`를 다듬으면서 실측으로 만난 재현을 여기에 고정한다.
//! 각 테스트는 **창 400×240 고정**에서 버튼의 `rect`를 읽어 *상대 위치*로만
//! 단언한다(절대 픽셀 비교 금지 — 여러 요소가 결합되면 값이 흔들린다).
//!
//! | # | 재현 | 관측 |
//! |---|---|---|
//! | 1a | `wrap: true` 행 + **버튼 직접** | 정상 — 3줄로 접힌다 |
//! | 1b | `wrap: true` 행 + **중첩 Row 자식** | **고침** — intrinsic 크기를 예약해 3줄(334.9 ≤ 400) |
//! | 1c | 한 단계 **더 깊은** 중첩 | **고침** — 깊이와 무관하게 접힌다 |
//! | 2 | 내용 크기 부모 안의 `width: fill` | **고침** — 교차축을 내용 폭으로 고정(형제가 212) |
//! | 2b | 같은 구조 + 부모에 `width` 선언 | 정상 — 형제가 200 뒤(212)에서 시작 |
//! | 2c | 같은 구조 + `fill`에 `max-width` | 정상 — 상한이 팽창을 막는다 |
//! | 3 | `justify: end` | 정상 — 밀 공간이 있는 컨테이너(모달)에서 동작한다 |
//!
//! 1b/1c/2는 0.8.1에서 고쳤다 — 단언은 **고친 뒤 동작**을 잠근다.

/// 재현 창 크기 — 모든 측정은 이 창 안에서의 상대값이다.
const W: f32 = 400.0;
const H: f32 = 240.0;

elm_magic::css! {
    .bug_root { padding: 0; }
    .bug_wrap { wrap: true; gap: 4; }
    .bug_line { gap: 4; }
    .bug_group { gap: 4; }
    .bug_btn { padding: 4 8; min-height: 26; }
    .bug_panel { min-width: 200; }
    .bug_rule { width: fill; height: 1; }
    .bug_end { justify: end; }
    .bug_end_fixed { width: 300; justify: end; }
    .bug_modal { padding: 8; gap: 4; }
    .bug_panel_fixed { width: 200; min-width: 200; }
    .bug_rule_capped { width: fill; max-width: 160; }
}

/// 긴 라벨 12개 — 합계 폭이 창(400)을 확실히 넘게 한다.
fn labels() -> Vec<String> {
    (0..12).map(|i| format!("Button {i:02}")).collect()
}

/// 중첩 그룹 3개(각 4개) — 셸의 `ToolBar`/`BarGroup`과 같은 형태.
fn groups() -> Vec<Vec<String>> {
    labels().chunks(4).map(|c| c.to_vec()).collect()
}

/// 한 단계 더 깊은 형태 — 그룹(4) 안에 줄(2)이 둘.
fn groups_deep() -> Vec<Vec<Vec<String>>> {
    groups()
        .into_iter()
        .map(|g| g.chunks(2).map(|c| c.to_vec()).collect())
        .collect()
}

elm_magic::view! {
    /// 재현 1a — `wrap` 행에 **버튼을 직접** 넣는다.
    fn WrapFlat() {
        let items = labels();
        <Col class="bug_root">
            <Row class="bug_wrap">
                {items.into_iter().map(|l| <Button class="bug_btn">"{l}"</Button>)}
            </Row>
        </Col>
    }

    /// 재현 1b — `wrap` 행의 자식이 **중첩 컨테이너**일 때.
    fn WrapNested() {
        let rows = groups();
        <Col class="bug_root">
            <Row class="bug_wrap">
                {rows.into_iter().map(|g| <Row class="bug_group">{g.into_iter().map(|l| <Button class="bug_btn">"{l}"</Button>)}</Row>)}
            </Row>
        </Col>
    }

    /// 재현 2 — 내용 크기 `Row` 안에서 내용 크기 `Col`의 자식이 `width: fill`.
    fn FillInsideContentParent() {
        <Col class="bug_root">
            <Row class="bug_line">
                <Col class="bug_panel">
                    <Col class="bug_rule" />
                </Col>
                <Button class="bug_btn">"sibling"</Button>
            </Row>
        </Col>
    }

    /// 재현 3a — 내용 크기 **창**(모달) 안의 `justify: end`.
    ///
    /// 두 줄을 넣는다: (a) 그냥, (b) `justify: end`. 창이 내용 크기라 밀 공간이 없다.
    fn JustifyInModal(open = true) {
        <Modal class="bug_modal" on_close={open = false}>
            <Row class="bug_line">
                <Button class="bug_btn">"plain 1"</Button>
                <Button class="bug_btn">"plain 2"</Button>
            </Row>
            <Row class="bug_line bug_end">
                <Button class="bug_btn">"end 1"</Button>
                <Button class="bug_btn">"end 2"</Button>
            </Row>
        </Modal>
    }

    /// 재현 3b — 같은 창에 **폭을 선언한** 행을 넣으면 그 폭 안에서 정렬이 먹는다.
    fn JustifyInWideModal(open = true) {
        <Modal class="bug_modal" on_close={open = false}>
            <Row class="bug_line">
                <Button class="bug_btn">"plain 1"</Button>
                <Button class="bug_btn">"plain 2"</Button>
            </Row>
            <Row class="bug_line bug_end_fixed">
                <Button class="bug_btn">"wide 1"</Button>
                <Button class="bug_btn">"wide 2"</Button>
            </Row>
        </Modal>
    }

    /// 재현 1c — 한 단계 더 깊은 중첩(그룹 안 줄)에서도 `wrap`이 무시되는가.
    fn WrapNestedTwice() {
        let rows = groups_deep();
        <Col class="bug_root">
            <Row class="bug_wrap">
                {rows.into_iter().map(|mid| <Row class="bug_group">{mid.into_iter().map(|line| <Row class="bug_line">{line.into_iter().map(|l| <Button class="bug_btn">"{l}"</Button>)}</Row>)}</Row>)}
            </Row>
        </Col>
    }

    /// 재현 2b — 부모에 **폭을 선언**하면 `fill`이 밖으로 새지 않는다(우회).
    fn FillInsideFixedParent() {
        <Col class="bug_root">
            <Row class="bug_line">
                <Col class="bug_panel_fixed">
                    <Col class="bug_rule" />
                </Col>
                <Button class="bug_btn">"sibling"</Button>
            </Row>
        </Col>
    }

    /// 재현 2c — `max-width`를 주면 `fill`의 팽창이 상한 안에서 멈춘다.
    fn FillInsideContentParentCapped() {
        <Col class="bug_root">
            <Row class="bug_line">
                <Col class="bug_panel">
                    <Col class="bug_rule_capped" />
                </Col>
                <Button class="bug_btn">"sibling"</Button>
            </Row>
        </Col>
    }
}
/// 한 프레임 렌더하고 **버튼 라벨 → rect**를 돌려준다 (elm-magic 계약만 사용).
fn buttons<C: elm_magic::Component>(props: &C::Props) -> Vec<(String, egui::Rect)> {
    let ctx = egui::Context::default();
    let mut elm = elm_magic::Ctx::default();
    let mut out: Vec<(String, egui::Rect)> = Vec::new();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(W, H),
        )),
        ..Default::default()
    };
    let mut response = ctx.run_ui(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let tree = elm_magic::frame::<C>(&mut elm, props);
            let pass = elm_magic_egui::render_with_palette(
                ui,
                &tree,
                &mut elm.arena,
                &elm_magic::style::Palette::dark(),
            );
            out = pass
                .buttons
                .iter()
                .map(|(label, resp)| (label.clone(), resp.rect))
                .collect();
        });
    });
    response.textures_delta.clear();
    out
}

/// 라벨로 rect 하나.
fn rect_of(rows: &[(String, egui::Rect)], label: &str) -> egui::Rect {
    let names: Vec<&String> = rows.iter().map(|(l, _)| l).collect();
    rows.iter()
        .find(|(l, _)| l == label)
        .unwrap_or_else(|| panic!("{label} 버튼이 없다: {names:?}"))
        .1
}

/// 행이 몇 줄로 접혔는가 — 서로 다른 `top` 값의 개수.
fn lines(rows: &[(String, egui::Rect)]) -> usize {
    let mut tops: Vec<i32> = rows.iter().map(|(_, r)| r.top() as i32).collect();
    tops.sort_unstable();
    tops.dedup();
    tops.len()
}
/// 재현 1a — `wrap: true` 행은 **줄바꿈해야** 한다(버튼을 직접 넣으면 정상).
#[test]
fn wrap_flat_row_wraps_within_the_window() {
    let rows = buttons::<WrapFlat>(&WrapFlatProps::default());
    assert_eq!(rows.len(), 12, "버튼 12개가 다 그려져야 한다");
    let right = rows.iter().map(|(_, r)| r.right()).fold(0.0, f32::max);
    println!(
        "1a flat wrap: lines={} max_right={right:.1} (window {W})",
        lines(&rows)
    );
    assert!(
        right <= W,
        "wrap: true인데 마지막 버튼이 창 밖({right:.1} > {W}) — 줄바꿈이 일어나지 않았다"
    );
    assert!(lines(&rows) > 1, "wrap: true인데 한 줄이다");
}

/// 재현 1b — 자식이 **중첩 컨테이너**여도 `wrap`이 동작한다.
///
/// egui의 줄바꿈 판단은 그리기 전에 알려진 크기로만 이뤄지므로, 어댑터가 컨테이너
/// 자식의 intrinsic 크기를 미리 예약한다 — 셸의 `ToolBar`/`BarGroup`도 이제 접힌다.
#[test]
fn wrap_applies_to_nested_container_children() {
    let flat = buttons::<WrapFlat>(&WrapFlatProps::default());
    let nested = buttons::<WrapNested>(&WrapNestedProps::default());
    let flat_right = flat.iter().map(|(_, r)| r.right()).fold(0.0, f32::max);
    let nested_right = nested.iter().map(|(_, r)| r.right()).fold(0.0, f32::max);
    println!(
        "1a flat: lines={} max_right={flat_right:.1} / 1b nested: lines={} max_right={nested_right:.1} (window {W})",
        lines(&flat),
        lines(&nested)
    );
    // 정상 동작 — 버튼을 직접 넣으면 접힌다.
    assert!(
        lines(&flat) > 1 && flat_right <= W,
        "1a: wrap이 동작해야 한다 (lines={}, right={flat_right:.1})",
        lines(&flat)
    );
    // 고침 — 중첩 컨테이너 자식에서도 접힌다.
    assert!(
        lines(&nested) > 1,
        "1b: 여러 줄로 접혀야 한다 (lines={})",
        lines(&nested)
    );
    assert!(
        nested_right <= W,
        "1b: 창 안({nested_right:.1} ≤ {W})이어야 한다"
    );
}

/// 재현 2 — 내용 크기 부모 안의 `width: fill`도 부모 폭 안에 머문다.
#[test]
fn width_fill_inside_content_parent_keeps_siblings_inside() {
    let rows = buttons::<FillInsideContentParent>(&FillInsideContentParentProps::default());
    let sibling = rect_of(&rows, "sibling");
    println!(
        "2 fill-in-content-parent: sibling.left={:.1} right={:.1} (panel min-width 200, window {W})",
        sibling.left(),
        sibling.right()
    );
    // 패널 폭(200)이 지켜지면 형제는 x≈212에서 시작하고 창 안에 머문다.
    assert!(
        sibling.left() > 200.0,
        "패널 폭(200) 뒤에서 시작해야 한다 (실측 {:.1})",
        sibling.left()
    );
    assert!(
        sibling.right() <= W,
        "형제가 창 안({:.1} ≤ {W})이어야 한다",
        sibling.right()
    );
}

/// 재현 3 — `justify: end`는 **밀 공간이 있으면 정상**이다(버그 아님).
///
/// 처음에 "모달에서 무효"라고 판단했던 것은 캡처 숫자를 잘못 읽은 것이었다 —
/// 액션 행은 실제로 오른쪽 끝(내용 우측 경계)에 붙어 있었다. 여기서 그 근거를 남긴다:
/// elm-magic 모달 창은 내용보다 넓어서 밀 공간이 있고, 그 안에서 `justify: end`가
/// 첫 버튼을 223px 오른쪽으로 민다.
#[test]
fn justify_end_works_where_there_is_spare_width() {
    let tight = buttons::<JustifyInModal>(&JustifyInModalProps::default());
    let plain = rect_of(&tight, "plain 1").left();
    let end = rect_of(&tight, "end 1").left();
    let wide_rows = buttons::<JustifyInWideModal>(&JustifyInWideModalProps::default());
    let wide_plain = rect_of(&wide_rows, "plain 1").left();
    let wide = rect_of(&wide_rows, "wide 1").left();
    println!(
        "3 modal: plain={plain:.1} end={end:.1} | width(300) 창: plain={wide_plain:.1} wide={wide:.1}"
    );
    // 모달 창은 내용보다 넓다 — 그래서 액션 행의 `justify: end`가 먹는다.
    assert!(
        end - plain > 100.0,
        "모달 안에서 `justify: end`가 밀어야 한다 (실측 {:.1})",
        end - plain
    );
    // 폭을 선언한 행도 그 폭 안에서 정렬된다(우회/대안).
    assert!(
        wide - wide_plain > 100.0,
        "폭을 준 행은 오른쪽으로 밀려야 한다 (실측 {:.1})",
        wide - wide_plain
    );
}

/// 재현 1c — 중첩이 한 단계 더 깊어도 `wrap`이 동작한다(1b와 같은 규칙).
#[test]
fn wrap_applies_to_deeper_nesting_too() {
    let nested = buttons::<WrapNestedTwice>(&WrapNestedTwiceProps::default());
    let right = nested.iter().map(|(_, r)| r.right()).fold(0.0, f32::max);
    println!(
        "1c nested-twice: lines={} max_right={right:.1} (window {W})",
        lines(&nested)
    );
    assert!(
        lines(&nested) > 1,
        "1c: 여러 줄로 접혀야 한다 (lines={})",
        lines(&nested)
    );
    assert!(right <= W, "1c: 창 안({right:.1} ≤ {W})이어야 한다");
}

/// 재현 2b — 부모에 `width`를 **선언**하면 `fill` 자식이 그 폭에 갇힌다.
///
/// 2는 이제 교차축을 내용 폭으로 고정해 고쳐졌지만, 폭을 선언하는 쪽이 더 확실한
/// 방법임을 함께 잠근다 — 형제가 200 뒤(212)에서 시작한다.
#[test]
fn width_fill_inside_declared_width_parent_stays_inside() {
    let rows = buttons::<FillInsideFixedParent>(&FillInsideFixedParentProps::default());
    let sibling = rect_of(&rows, "sibling");
    println!(
        "2b fill-in-fixed-parent: sibling.left={:.1} right={:.1} (panel width 200)",
        sibling.left(),
        sibling.right()
    );
    assert!(
        sibling.left() >= 200.0,
        "부모 폭(200) 뒤에서 시작해야 한다 (실측 {:.1})",
        sibling.left()
    );
    assert!(
        sibling.right() <= W,
        "형제가 창 안에 있어야 한다 (실측 {:.1} > {W})",
        sibling.right()
    );
}

/// 재현 2c — `max-width`가 `fill`의 팽창을 상한 안으로 묶는다(우회).
///
/// `child_budgets`가 `max-*` 상한에 닿은 자식을 먼저 확정하므로, `fill`에
/// `max-width`를 주면 부모가 창 전체를 먹지 않는다.
#[test]
fn max_width_caps_fill_inside_content_parent() {
    let rows =
        buttons::<FillInsideContentParentCapped>(&FillInsideContentParentCappedProps::default());
    let sibling = rect_of(&rows, "sibling");
    println!(
        "2c fill-capped: sibling.left={:.1} right={:.1} (panel min-width 200, cap 160)",
        sibling.left(),
        sibling.right()
    );
    assert!(
        sibling.left() >= 200.0,
        "패널 min-width(200)가 지켜져야 한다 (실측 {:.1})",
        sibling.left()
    );
    assert!(
        sibling.right() <= W,
        "형제가 창 안에 있어야 한다 (실측 {:.1} > {W})",
        sibling.right()
    );
}
