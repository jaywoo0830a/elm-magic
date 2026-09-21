//! 계획 층 계약 — **리눅스/macOS에서도 돈다** (windows-reactor 의존 없음).
//!
//! 어댑터의 매핑 규칙(`Element` → WinUI 컨트롤)을 여기서 고정한다. WinUI 층은
//! 이 계획을 컨트롤로 바꾸는 얇은 코드이므로, 매핑이 틀어지면 이 테스트가 먼저
//! 깨진다.

use elm_magic_windows_reactor::{plan, Pass, PlanEvent, PlanKind, PlanNode, Severity};

/// 자식 중 특정 컨트롤을 찾는다 (없으면 panic).
fn child<'a>(node: &'a PlanNode, control: &str) -> &'a PlanNode {
    node.children
        .iter()
        .find(|c| c.control() == control)
        .unwrap_or_else(|| panic!("{control} 자식이 없다: {:?}", kinds(node)))
}

fn kinds(node: &PlanNode) -> Vec<&'static str> {
    node.children.iter().map(|c| c.control()).collect()
}

#[test]
fn structure_maps_to_winui_controls() {
    let tree = elm_magic::ui! {
        <Col>
            <Strong>"제목"</Strong>
            <Text>"본문"</Text>
            <Progress value={0.25} />
            <Spinner />
            <Divider />
            <Banner kind="warn">"조심"</Banner>
            <>
                <Td>"셀"</Td>
                <Th>"정적 헤더"</Th>
            </>
        </Col>
    };

    let (node, pass) = plan(&tree);
    assert_eq!(node.control(), "StackPanel");
    // `<>`는 레이아웃 자식 위치에서 **매크로가 펼친다** — 트리에 Fragment 노드가
    // 남지 않으므로 여기서도 자식이 그대로 이어 붙는다(순서 보존).
    assert_eq!(
        kinds(&node),
        vec![
            "TextBlock", // Strong
            "TextBlock", // Text
            "ProgressBar",
            "ProgressRing",
            "Border",    // Divider
            "InfoBar",   // Banner
            "TextBlock", // <>
            "TextBlock", // <> (Th 정적 → 굵은 텍스트)
        ]
    );

    // Strong은 굵게, Th(정적)는 굵은 텍스트.
    let strong = child(&node, "TextBlock");
    assert!(matches!(strong.kind, PlanKind::Text { strong: true }));
    assert!(matches!(
        node.children.last().expect("헤더").kind,
        PlanKind::Text { strong: true }
    ));
    assert_eq!(node.children.last().expect("헤더").text, "정적 헤더");

    // 계획 요약(Pass) — 컨트롤 종류와 텍스트가 순서대로 남는다.
    assert_eq!(pass.count("StackPanel"), 1);
    assert_eq!(pass.count("TextBlock"), 4); // 제목/본문/셀/헤더
    assert!(pass.has_text("조심"));
    assert_eq!(pass.inputs, 0);
    assert_eq!(pass.dialogs, 0);
}

// 본문 **전체**가 `<>`면 트리에 Fragment 노드가 남는다 (자식 위치와 다른 점).
elm_magic::view! {
    fn Body() {
        <>
            <Text>"x"</Text>
            <Text>"y"</Text>
        </>
    }
}

#[test]
fn a_fragment_body_maps_to_a_reactor_fragment() {
    let mut ctx = elm_magic::Ctx::new();
    let tree = elm_magic::frame::<Body>(&mut ctx, &BodyProps::default());
    let (node, pass) = plan(&tree);

    assert!(matches!(node.kind, PlanKind::Fragment));
    assert_eq!(kinds(&node), vec!["TextBlock", "TextBlock"]);
    assert_eq!(pass.count("Fragment"), 1);
}

#[test]
fn progress_is_clamped_to_the_unit_range() {
    let (node, _) = plan(&elm_magic::ui! { <Progress value={1.5} /> });
    assert_eq!(node.fraction, Some(1.0));

    let (node, _) = plan(&elm_magic::ui! { <Progress value={-2.0} /> });
    assert_eq!(node.fraction, Some(0.0));
}

#[test]
fn banner_kind_becomes_a_severity() {
    let cases = [
        ("error", Severity::Error),
        ("warn", Severity::Warning),
        ("warning", Severity::Warning),
        ("success", Severity::Success),
        ("ok", Severity::Success),
        ("info", Severity::Info),
        ("무엇이든", Severity::Info),
    ];
    for (kind, expected) in cases {
        assert_eq!(Severity::from_kind(kind), expected, "kind={kind}");
    }

    let (node, _) = plan(&elm_magic::ui! { <Banner kind="error">"실패"</Banner> });
    assert!(matches!(
        node.kind,
        PlanKind::Banner {
            severity: Severity::Error
        }
    ));
    assert_eq!(node.text, "실패");
}

#[test]
fn raw_closure_is_carried_with_its_payload_contract() {
    let tree = elm_magic::ui! { <Raw>|payload: &mut String| { payload.push_str("hit"); }</Raw> };
    let (node, pass) = plan(&tree);

    // 계획이 클로저를 그대로 들고 있다 — WinUI 층은 `&mut Option<View>`를 넣는다.
    let widget = node.raw.clone().expect("<Raw> 클로저");
    assert_eq!(pass.count("Raw"), 1);

    let mut via_tree = String::new();
    elm_magic::raw::invoke(&tree, &mut via_tree);
    assert_eq!(via_tree, "hit");

    let mut via_plan = String::new();
    widget(&mut via_plan);
    assert_eq!(via_plan, "hit");
}

// ── 컴포넌트 전체 파이프라인 (frame → plan) ──────────────────

elm_magic::view! {
    fn Form(
        name = String::new(),
        agreed = false,
        pct = 0.5,
        hits = 0,
    ) {
        <Col>
            <Input value={name.clone()} on_change={name = _} on_enter={hits += 1} />
            <TextArea value={name.clone()} on_change={name = _} />
            <Check checked={agreed} on_change={agreed = _}>"동의"</Check>
            <Button on_click={hits += 1}>"저장"</Button>
            <Button disabled={true} on_click={hits += 1}>"삭제"</Button>
            <Progress value={pct} />
            <Tab active={true} on_click={hits += 1}>"Home"</Tab>
            <Th on_click={hits += 1}>"정렬"</Th>
            <Modal title="확인" on_close={hits += 1}>
                <Text>"본문"</Text>
            </Modal>
            <Col on_click={hits += 1}>
                <Text>"카드"</Text>
            </Col>
        </Col>
    }
}

/// 한 프레임을 돌려 계획을 만든다 (WinUI 층이 `view()`에서 하는 일의 절반).
fn form_plan(props: &FormProps) -> (PlanNode, Pass) {
    let mut ctx = elm_magic::Ctx::new();
    let tree = elm_magic::frame::<Form>(&mut ctx, props);
    plan(&tree)
}

#[test]
fn interactive_widgets_carry_state_and_events() {
    let props = FormProps {
        agreed: Some(true),
        pct: Some(0.75),
        ..Default::default()
    };
    let (node, pass) = form_plan(&props);

    // 입력 2개 (Input + TextArea) — 포커스/값 계약의 근거.
    assert_eq!(pass.inputs, 2);
    assert_eq!(pass.count("TextBox"), 2);
    assert_eq!(pass.count("ProgressBar"), 1);
    assert_eq!(pass.count("ContentDialog"), 1);
    assert_eq!(pass.dialogs, 1);

    // Input: 값 + 변경 핸들러 + Enter 핸들러(계획에만 남는다).
    let input = child(&node, "TextBox");
    assert!(matches!(input.kind, PlanKind::TextBox { multiline: false }));
    assert_eq!(input.value.as_deref(), Some(""));
    assert!(matches!(input.event, Some(PlanEvent::Change(_))));
    assert!(input.enter.is_some(), "on_enter는 계획에 남아야 한다");

    // TextArea는 여러 줄.
    let area = node
        .children
        .iter()
        .filter(|c| c.control() == "TextBox")
        .nth(1)
        .expect("TextArea");
    assert!(matches!(area.kind, PlanKind::TextBox { multiline: true }));

    // Check: 체크 상태 + 토글 핸들러 + 라벨.
    let check = child(&node, "CheckBox");
    assert_eq!(check.checked, Some(true));
    assert!(matches!(check.event, Some(PlanEvent::Toggle(_))));
    assert_eq!(check.text, "동의");

    // Progress: 0..1 그대로 (WinUI 층이 0..100으로 바꾼다).
    assert_eq!(child(&node, "ProgressBar").fraction, Some(0.75));
}

#[test]
fn buttons_tabs_headers_and_containers_become_clicks() {
    let (node, pass) = form_plan(&FormProps::default());

    let buttons: Vec<&PlanNode> = node
        .children
        .iter()
        .filter(|c| c.control() == "Button")
        .collect();
    // 저장 / 삭제(disabled) / Tab / Th(on_click)
    assert_eq!(buttons.len(), 4);
    assert!(!buttons[0].disabled && matches!(buttons[0].event, Some(PlanEvent::Click(_))));
    assert!(
        buttons[1].disabled,
        "disabled 버튼은 is_enabled(false)로 간다"
    );
    assert!(buttons[2].active, "Tab의 active는 계획에 남는다");

    // 라벨은 순서대로 남는다 — E2E 회귀에 쓴다.
    assert_eq!(pass.labeled, vec!["동의", "저장", "삭제", "Home", "정렬"]);
}

#[test]
fn modal_becomes_a_dialog_with_a_close_event() {
    let (node, _) = form_plan(&FormProps::default());
    let modal = child(&node, "ContentDialog");
    assert_eq!(modal.text, "확인");
    assert!(matches!(modal.event, Some(PlanEvent::Close(_))));
    assert_eq!(kinds(modal), vec!["TextBlock"]);
}

#[test]
fn a_clickable_container_keeps_its_children() {
    let (node, _) = form_plan(&FormProps::default());
    // 마지막 자식 = 클릭 가능한 <Col> (StackPanel + Click).
    let card = node.children.last().expect("카드");
    assert!(matches!(card.kind, PlanKind::Stack { vertical: true }));
    assert!(matches!(card.event, Some(PlanEvent::Click(_))));
    assert_eq!(card.children[0].text, "카드");
}
