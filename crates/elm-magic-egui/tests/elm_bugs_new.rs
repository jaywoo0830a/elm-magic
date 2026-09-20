//! elm-magic 최소 재현 — **freedf-gui 코드를 쓰지 않는다**(elm-magic + egui만).
//!
//! `src/style.rs`/`ui/*`를 다듬으면서 실측으로 만난 재현을 여기에 고정한다.
//! 각 테스트는 **창 400×240 고정**에서 버튼의 `rect`를 읽어 *상대 위치*로만
//! 단언한다(절대 픽셀 비교 금지 — 여러 요소가 결합되면 값이 흔들린다).
//!
//! `elm_wrap_tests.rs`와 **겹치지 않는** 재현만 남긴다 — `wrap` 1a/1b/1c,
//! 내용 크기 부모의 `fill` 2/2b/2c, `justify: end` 3은 그쪽 파일에 있다.
//!
//! | # | 재현 | 관측 |
//! |---|---|---|
//! | 4 | `flex-grow: 1` — 텍스트 / 빈 컨테이너 / 위젯 | **고침** — 셋 다 행 끝(243/245/247) |
//! | 5 | `justify: center/end` — 폭 선언 / `fill` / 클래스 충돌 | 정상 — 단 **클래스가 다투면 나중 선언이 이긴다** |
//! | 5e | 셸 패널과 같은 조건(패딩·`<If>`·`overflow`) | 정상 — 넷 다 가운데 |
//! | 6 | 행 안의 `align-self: center` | **고침** — 행 높이 안에서 정렬해 다음 행이 top=24 |
//!
//! 4/6은 0.8.1에서 고쳤다 — 단언은 **고친 뒤 동작**을 잠근다(되돌아가면 깨진다).

/// 재현 창 크기 — 모든 측정은 이 창 안에서의 상대값이다.
const W: f32 = 400.0;
const H: f32 = 240.0;

elm_magic::css! {
    .bug_root { padding: 0; }
    .bug_line { gap: 4; }
    .bug_btn { padding: 4 8; min-height: 26; }
    .bug_end { justify: end; }
    .bug_end_fixed { width: 300; justify: end; }
    // 재현 4 — 같은 폭(300)에서 첫 자식의 종류만 바꾼다.
    .bug_grow_row { width: 300; gap: 4; }
    .bug_grow_text { flex-grow: 1; }
    .bug_grow_box { flex-grow: 1; }
    .bug_grow_btn { flex-grow: 1; }
    // 재현 5 — 정렬은 부모가 폭을 줘야 의미가 생긴다.
    .bug_center_fixed { width: 300; justify: center; }
    .bug_center_fill { width: fill; justify: center; }
    // 재현 6 — 교차축 정렬.
    .bug_self_center { align-self: center; }
    // 재현 5d — 같은 `justify`를 다투는 두 번째 클래스(선언 순서 실험용).
    .bug_end2 { justify: end; }
    // 재현 5e — 셸의 패널과 같은 **폭을 선언한 컬럼**.
    .bug_col_fixed { width: 216; }
    // 재현 5e-2 — 패널의 패딩까지 같게(패딩이 있으면 `fill`의 기준이 달라지는가).
    .bug_col_pad { width: 216; padding: 12 8; }
    // 재현 5e-4 — 패널의 `overflow: hidden`.
    .bug_col_clip { width: 216; padding: 12 8; overflow: hidden; }
}

elm_magic::view! {
    /// 재현 4 — `flex-grow: 1`이 텍스트/컨테이너/위젯 **모두**에서 먹는지.
    ///
    /// 세 행 모두 폭 300이고 **첫 자식의 종류만** 다르다. 마지막 버튼의 x가 곧
    /// "grow가 남는 폭을 먹었는가"다 — 셸의 패널 머리(제목 + 개수 배지)에서 만난 형태.
    fn FlexGrowKinds() {
        <Col class="bug_root">
            <Row class="bug_grow_row">
                <Text class="bug_grow_text">"label"</Text>
                <Button class="bug_btn">"tail-text"</Button>
            </Row>
            <Row class="bug_grow_row">
                <Row class="bug_grow_box" />
                <Button class="bug_btn">"tail-box"</Button>
            </Row>
            <Row class="bug_grow_row">
                <Button class="bug_grow_btn">"head-btn"</Button>
                <Button class="bug_btn">"tail-btn"</Button>
            </Row>
        </Col>
    }

    /// 재현 5 — 같은 `justify: center`라도 **행의 폭이 어떻게 정해졌는가**가 결과를 가른다.
    ///
    /// 5a: `width: 300` + center → 가운데. 5b: `width: fill` + center → 창 기준 가운데.
    /// 5c: `width: 300` + end → 오른쪽 끝. 5d: 두 클래스가 `justify`를 **다투면**
    /// 어느 쪽이 이기는지(선언 순서가 앞/뒤인 클래스를 나란히 둔다).
    fn JustifyCenterKinds() {
        <Col class="bug_root">
            <Row class="bug_center_fixed">
                <Button class="bug_btn">"c-fixed"</Button>
            </Row>
            <Row class="bug_center_fill">
                <Button class="bug_btn">"c-fill"</Button>
            </Row>
            <Row class="bug_end_fixed">
                <Button class="bug_btn">"e-fixed"</Button>
            </Row>
            <Row class="bug_center_fixed bug_end">
                <Button class="bug_btn">"x-early"</Button>
            </Row>
            <Row class="bug_center_fixed bug_end2">
                <Button class="bug_btn">"x-late"</Button>
            </Row>
        </Col>
    }

    /// 재현 5e — 셸 패널과 **같은 조건**을 하나씩 쌓아 본다(어느 조건이 정렬을 죽이는가).
    ///
    /// 5e-1 고정 폭 컬럼 / 5e-2 패딩 있는 고정 폭 컬럼 / 5e-3 `<If>` 안 /
    /// 5e-4 `overflow: hidden` 컬럼 — 모두 같은 `justify: center` + `width: fill` 행이다.
    fn JustifyInsidePanelLike() {
        <Col class="bug_root">
            <Col class="bug_col_fixed">
                <Row class="bug_center_fill">
                    <Button class="bug_btn">"c-fixed-col"</Button>
                </Row>
            </Col>
            <Col class="bug_col_pad">
                <Row class="bug_center_fill">
                    <Button class="bug_btn">"c-pad-col"</Button>
                </Row>
            </Col>
            <Col class="bug_col_pad">
                <If when={true}>
                    <Row class="bug_center_fill">
                        <Button class="bug_btn">"c-in-if"</Button>
                    </Row>
                </If>
            </Col>
            <Col class="bug_col_clip">
                <Row class="bug_center_fill">
                    <Button class="bug_btn">"c-clip"</Button>
                </Row>
            </Col>
        </Col>
    }

    /// 재현 6 — 행 안의 `align-self: center`(행 높이 안에서 정렬된다).
    fn AlignSelfCenterRow() {
        <Col class="bug_root">
            <Row class="bug_line">
                <Text class="bug_self_center">"centered"</Text>
            </Row>
            <Row class="bug_line">
                <Button class="bug_btn">"after"</Button>
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

/// 재현 4 — `flex-grow: 1`이 **텍스트 노드에서도** 먹는다.
///
/// 세 행 모두 `width: 300`이고 첫 자식만 다르다. 마지막 버튼의 x가 곧 "남는 폭을
/// 먹었는가"다: 위젯 / 빈 컨테이너 / 텍스트 셋 다 행 끝으로 간다(고침).
#[test]
fn flex_grow_works_on_text_nodes() {
    let rows = buttons::<FlexGrowKinds>(&FlexGrowKindsProps::default());
    let text = rect_of(&rows, "tail-text").left();
    let boxed = rect_of(&rows, "tail-box").left();
    let widget = rect_of(&rows, "tail-btn").left();
    println!("4 flex-grow(폭 300): text={text:.1} container={boxed:.1} button={widget:.1}");
    // 위젯과 컨테이너는 남는 폭을 먹는다 — 뒤 형제가 행 끝으로 간다.
    assert!(widget > 200.0, "위젯 grow가 무시됐다 (실측 {widget:.1})");
    assert!(
        boxed > 200.0,
        "컨테이너 grow가 무시됐다 (실측 {boxed:.1}) — 스페이서 우회가 안 먹는다"
    );
    // 고침 — 텍스트도 남는 폭을 예약해 뒤 버튼이 행 끝으로 간다.
    assert!(text > 200.0, "텍스트 grow가 먹어야 한다 (실측 {text:.1})");
}

/// 재현 5 — `justify`의 결과는 **폭이 어떻게 정해졌는가**에 달렸다.
///
/// 5a `width: 300` + center → 가운데. 5b `width: fill` + center → 창 기준 가운데.
/// 5c `width: 300` + end → 오른쪽 끝. 5d 두 클래스가 `justify`를 다툴 때의 승자.
#[test]
fn justify_aligns_where_the_container_has_width() {
    let rows = buttons::<JustifyCenterKinds>(&JustifyCenterKindsProps::default());
    let fixed = rect_of(&rows, "c-fixed").left();
    let fill = rect_of(&rows, "c-fill").left();
    let end = rect_of(&rows, "e-fixed").left();
    let early = rect_of(&rows, "x-early").left();
    let late = rect_of(&rows, "x-late").left();
    println!(
        "5 justify(창 {W}): center-fixed={fixed:.1} center-fill={fill:.1} end-fixed={end:.1} \
         충돌(x-early={early:.1} x-late={late:.1})"
    );
    // 5a — 폭 300 안에서 가운데(버튼 폭 ≈ 41 → (300-41)/2 ≈ 129).
    assert!(
        fixed > 100.0,
        "`width: 300` 행의 center가 무시됐다 (실측 {fixed:.1})"
    );
    // 5c — 폭 300 안에서 오른쪽 끝(≈ 259).
    assert!(
        end > 200.0,
        "`width: 300` 행의 end가 무시됐다 (실측 {end:.1})"
    );
    // 5b — `fill` 행은 창 폭(400) 기준 가운데(≈ 179)로 간다.
    assert!(
        fill > 100.0,
        "`width: fill` 행의 center가 무시됐다 (실측 {fill:.1})"
    );
    // 5d — 두 클래스가 다투면 **CSS에서 나중에 선언된 쪽**이 이긴다(실측).
    //     `.bug_end`(앞 선언)는 center에 밀렸고 `.bug_end2`(뒤 선언)는 이겼다.
    assert!(
        early < 200.0 && late > 200.0,
        "선언 순서 규칙이 바뀌었다: x-early={early:.1}(center 기대) x-late={late:.1}(end 기대)"
    );
}

/// 재현 6 — 행 안의 `align-self: center`가 **행 높이 안**에서 정렬된다.
///
/// 가로 행의 교차축(세로) 정렬은 행 높이(자식들의 최대 높이)를 기준으로 하므로
/// 남은 세로를 먹지 않는다 — 다음 행이 바로 뒤에 온다(고침).
#[test]
fn align_self_center_stays_within_the_row() {
    let rows = buttons::<AlignSelfCenterRow>(&AlignSelfCenterRowProps::default());
    let after = rect_of(&rows, "after").top();
    println!("6 align-self: center 다음 행 top={after:.1} (창 높이 {H})");
    // 고침 — 다음 행이 첫 행 높이 뒤(≈ 24)에 온다.
    assert!(
        after < 100.0,
        "다음 행이 행 높이 뒤에 와야 한다 (실측 {after:.1})"
    );
}
/// 재현 5e — 셸 패널과 같은 조건에서 `justify: center` + `width: fill`이 살아 있는가.
///
/// 5e-1 고정 폭 / 5e-2 패딩 / 5e-3 `<If>` 안 / 5e-4 `overflow: hidden` — 같은 행을
/// 한 조건씩 쌓아 어느 것이 정렬을 죽이는지 가른다(셸의 빈 상태 안내문이 왼쪽에 붙었다).
#[test]
fn justify_center_inside_a_declared_width_column() {
    let rows = buttons::<JustifyInsidePanelLike>(&JustifyInsidePanelLikeProps::default());
    let fixed = rect_of(&rows, "c-fixed-col").left();
    let pad = rect_of(&rows, "c-pad-col").left();
    let in_if = rect_of(&rows, "c-in-if").left();
    let clip = rect_of(&rows, "c-clip").left();
    println!("5e 패널 유사 조건: 고정={fixed:.1} 패딩={pad:.1} If안={in_if:.1} clip={clip:.1}");
    // 넷 다 가운데(≈ 60~90)여야 한다 — 왼쪽(≈ 8~24)이면 그 조건이 정렬을 죽인 것이다.
    assert!(fixed > 40.0, "고정 폭 컬럼에서 정렬이 죽었다: {fixed:.1}");
    assert!(pad > 40.0, "패딩 컬럼에서 정렬이 죽었다: {pad:.1}");
    assert!(in_if > 40.0, "`<If>` 안에서 정렬이 죽었다: {in_if:.1}");
    assert!(
        clip > 40.0,
        "`overflow: hidden` 컬럼에서 정렬이 죽었다: {clip:.1}"
    );
}
