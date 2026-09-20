//! elm-magic **버그/계약 위반 사례 모음** — 최소 재현.
//!
//! 이 파일은 우리 앱의 기능 테스트가 아니다. 셸을 만들면서 elm-magic에서
//! **문서/직관과 다르게 동작하는 것**을 찾을 때마다 여기에 최소 재현을 남긴다.
//!
//! 재현에 필요한 CSS·컴포넌트는 **전부 이 파일 안에** 있어야 한다 — 외부 앱
//! 크레이트에 의존하지 않는다(과거 `freedf_gui`의 `PanelRow` atom을 참조해
//! 이 파일이 컴파일조차 되지 않았다).
//!
//! ## 사례 (0.8.4)
//!
//! | # | 사례 | 상태 |
//! |---|---|---|
//! | 1 | `Modal`의 `title`이 텍스트 수집에 없어 `assert_text(제목)` 실패 | **수정됨** — [`modal_title_is_collected_as_text`] |
//! | 2 | `click(라벨)`이 다중 텍스트 컨테이너를 못 찾음 | **수정됨** — [`click_finds_container_by_child_label`] |
//!
//! ## 우리 코드에서 제거한 우회
//!
//! - 사례 1 → 셸 테스트는 이제 **창 제목**으로 단언할 수 있다(본문 라벨 우회 불필요).
//! - 사례 2 → 셸 행은 이제 `click("제목")`으로 눌린다. `click_sel(&Selector::class(...))`는
//!   **트리 순서 첫 일치**라 엉뚱한 행을 눌릴 위험이 있으므로 라벨 클릭이 더 안전하다.

use std::cell::Cell;

// 클릭 콜백 발화 횟수 카운터 — `thread_local!`은 매크로 호출이라 doc 주석을 못 붙인다.
thread_local! {
    /// 클릭 가능한 `Row`의 콜백이 발화한 횟수.
    static ROW_CLICKS: Cell<usize> = const { Cell::new(0) };
    /// 중첩 클릭 프로브: 안쪽 `Button` 두 개의 발화 횟수.
    static INNER_A: Cell<usize> = const { Cell::new(0) };
    static INNER_B: Cell<usize> = const { Cell::new(0) };
}

/// 행 콜백이 불렸는지 세는 훅.
fn probe_row_click() {
    ROW_CLICKS.with(|c| c.set(c.get() + 1));
}

/// 안쪽 버튼 훅 — 바깥 행이 가로채면 이 값이 오르지 않는다.
fn probe_inner_a() {
    INNER_A.with(|c| c.set(c.get() + 1));
}

fn probe_inner_b() {
    INNER_B.with(|c| c.set(c.get() + 1));
}

elm_magic::view! {
    /// `Modal`(=`Dialog`의 원형)의 `title`이 텍스트 수집에 잡히는지 보는 프로브.
    pub fn ModalProbe() {
        <Modal class="modal" title="PROBE TITLE" on_close={probe_row_click()}>
            <Text class="text">"PROBE BODY"</Text>
        </Modal>
    }

    /// `on_click: fn()`을 가진 `Row` — 하네스가 누를 수 있는지 보는 프로브.
    pub fn RowProbe() {
        <Row class="panel__row" on_click={probe_row_click()}>
            <Text class="text">"PROBE ROW"</Text>
        </Row>
    }

    /// 셸의 atom(`PanelRow`) 자리 — 외부 앱 크레이트에 의존하지 않도록 같은 구조
    /// (제목 + 부가값 + `on_click`)를 로컬 view로 재현한다.
    pub fn PanelRowProbe() {
        <Row class="panel__row" on_click={probe_row_click()}>
            <Text class="panel__text">"PROBE ROW"</Text>
            <Text class="panel__meta">"meta"</Text>
        </Row>
    }

    /// 클릭 가능한 `Row` + 기본 `Text` + 두 번째 텍스트(meta 자리).
    pub fn TwoTextRowProbe() {
        <Row class="panel__row" on_click={probe_row_click()}>
            <Text class="text">"PROBE ROW"</Text>
            <Text class="text">"meta"</Text>
        </Row>
    }

    /// 눌리는 컨테이너 **안에** 눌리는 자식(`Button`)이 있는 경우 —
    /// 컨테이너가 안쪽 버튼 클릭을 가로채면 안 된다.
    pub fn NestedClickProbe() {
        <Row class="panel__row" on_click={probe_row_click()}>
            <Button on_click={probe_inner_a()}>"inner-a"</Button>
            <Button on_click={probe_inner_b()}>"inner-b"</Button>
        </Row>
    }
}

// ── 1. (수정됨) 모달 제목은 이제 텍스트 수집 대상이다 ────────────

/// `Modal`/`Dialog`의 `title`은 화면에 보이는 텍스트다(어댑터가 헤딩으로 그린다).
///
/// 예전에는 `ModalEl::texts_into`가 `[modal]` 마커와 자식만 모으고 제목을 빼서
/// `text()`/`assert_text(제목)`이 실패했다. 이제 제목도 수집한다.
#[test]
fn modal_title_is_collected_as_text() {
    let app = elm_magic::mount!(ModalProbe);
    assert!(
        app.render_tree().contains("PROBE TITLE"),
        "제목이 트리 라벨로는 존재한다 — 트리 덤프 규약이 바뀌었다"
    );
    assert!(
        app.a11y_tree().contains("PROBE TITLE"),
        "제목이 a11y 이름으로는 존재한다 — 접근성 트리 규약이 바뀌었다"
    );

    // 본문도 제목도 텍스트 수집에 들어온다 (예전엔 제목만 빠졌다).
    app.assert_text("PROBE BODY");
    app.assert_text("PROBE TITLE");
    assert!(
        app.text().contains("PROBE TITLE"),
        "제목이 텍스트 수집에서 빠졌다 — `ModalEl::texts_into` 회귀.\n수집된 텍스트:\n{}",
        app.text()
    );
}

// ── 2. (수정됨) 컨테이너 클릭은 자식 라벨로도 찾는다 ───────────────

/// `TestApp::click(label)`은 클릭 가능한 요소를 찾을 때 (`find_click`, `testing.rs`)
/// - `Button`/`Tab`/`ColumnHeader`(leaf) → `label() == text` **정확 일치**
/// - 그 외(컨테이너) → `subtree_text() == text` **전체 일치**  ← 예전 규칙
///
/// 를 요구했다. 그래서 자식 텍스트가 둘 이상인 행(제목 + 부가값 — 우리 `PanelRow`)은
/// 서브트리 텍스트가 이어 붙어 라벨과 달라져 `click("제목")`으로 **찾을 수 없었다**.
/// 이제 컨테이너는 **자식 텍스트 노드 중 하나가 라벨과 정확히 일치**하면 찾는다.
///
/// 단, 컨테이너 안에 눌리는 요소가 있으면 그쪽이 더 구체적이므로 컨테이너는 양보한다.
#[test]
fn click_finds_container_by_child_label() {
    // (a) 텍스트가 하나면 서브트리 텍스트 == 라벨 → 그대로 찾는다.
    ROW_CLICKS.with(|c| c.set(0));
    let mut single = elm_magic::mount!(RowProbe);
    single.click("PROBE ROW");
    assert_eq!(
        ROW_CLICKS.with(|c| c.get()),
        1,
        "텍스트가 하나인 클릭 가능한 Row는 click(라벨)로 눌려야 한다"
    );

    // (b) 텍스트가 둘(제목 + 부가값)이어도 제목 라벨로 찾는다 — 예전엔 못 찾았다.
    ROW_CLICKS.with(|c| c.set(0));
    let mut multi = elm_magic::mount!(TwoTextRowProbe);
    let subtree = multi.element().subtree_text();
    assert_ne!(
        subtree, "PROBE ROW",
        "서브트리 텍스트가 라벨과 같아졌다 — 이 테스트의 전제가 바뀌었다"
    );
    multi.click("PROBE ROW");
    assert_eq!(
        ROW_CLICKS.with(|c| c.get()),
        1,
        "다중 텍스트 행이 click(제목)로 눌려야 한다. 서브트리 텍스트: {subtree:?}"
    );

    // (b') 부가값 라벨도 자식 텍스트이므로 눌린다.
    ROW_CLICKS.with(|c| c.set(0));
    multi.click("meta");
    assert_eq!(
        ROW_CLICKS.with(|c| c.get()),
        1,
        "부가값 라벨도 자식 텍스트다 — click으로 눌려야 한다"
    );

    // (c) 우리 atom(`PanelRow`) 자리도 같은 규칙을 따른다 — 셸 행의 실제 형태.
    ROW_CLICKS.with(|c| c.set(0));
    let mut row = elm_magic::mount!(PanelRowProbe);
    row.click("PROBE ROW");
    assert_eq!(
        ROW_CLICKS.with(|c| c.get()),
        1,
        "PanelRow 구조(제목 + 부가값 + on_click)가 click(제목)로 눌려야 한다"
    );

    // (d) 컨테이너 안에 눌리는 요소가 있으면 바깥 행이 가로채지 않는다.
    ROW_CLICKS.with(|c| c.set(0));
    INNER_A.with(|c| c.set(0));
    INNER_B.with(|c| c.set(0));
    let mut nested = elm_magic::mount!(NestedClickProbe);
    nested.click("inner-a");
    assert_eq!(INNER_A.with(|c| c.get()), 1, "안쪽 Button이 눌려야 한다");
    assert_eq!(
        ROW_CLICKS.with(|c| c.get()),
        0,
        "바깥 행이 안쪽 버튼 클릭을 가로챘다 — find_click의 컨테이너 양보 규칙 회귀"
    );
    nested.click("inner-b");
    assert_eq!(
        INNER_B.with(|c| c.get()),
        1,
        "두 번째 안쪽 Button도 눌려야 한다"
    );
    assert_eq!(
        ROW_CLICKS.with(|c| c.get()),
        0,
        "바깥 행은 여전히 조용해야 한다"
    );
}
