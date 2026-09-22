//! 계획(plan) 층 — **플랫폼 독립**.
//!
//! `Element` 트리를 WinUI 컨트롤로 바꾸기 전에 "무엇을 그릴 것인가"를 중간 표현
//! ([`PlanNode`])으로 한 번 옮긴다. 이 층은 windows-reactor에 의존하지 않으므로
//! **리눅스/macOS에서도 컴파일·테스트된다** — 매핑 규칙은 여기서 고정하고,
//! WinUI 코드는 계획을 컨트롤로 바꾸는 얇은 층(`crate::winui`)만 남긴다.
//!
//! ## 스타일은 옮기지 않는다
//!
//! `Element::class`도 `style::resolve`도 호출하지 않는다. `css!`로 쓴 규칙은 이
//! 백엔드에서 효과가 없고, 색·간격·정렬은 WinUI 테마/리소스가 담당한다.
//! WinUI 수준 조정이 필요하면 `<Raw>`가 정식 통로다([`PlanNode::raw`]).
//!
//! ## 매핑표
//!
//! | elm-magic | 계획 | WinUI 컨트롤 |
//! |---|---|---|
//! | `<Col>` / `<Row>` | [`PlanKind::Stack`] | `StackPanel` (orientation) |
//! | `on_click`이 있는 컨테이너 | `Stack` + [`PlanEvent::Click`] | `Border::on_pointer_released` |
//! | `<Text>` / `<Strong>` | [`PlanKind::Text`] | `TextBlock` (Strong은 굵게) |
//! | `<Button>` | [`PlanKind::Button`] | `Button` |
//! | `<Tab>` | `Button` (+ [`PlanNode::active`]) | `Button` (문서 참고) |
//! | `<Th>` | 클릭 가능하면 `Button`, 아니면 굵은 `Text` | `Button` / `TextBlock` |
//! | `<Td>` | `Text` | `TextBlock` |
//! | `<Input>` / `<TextArea>` | [`PlanKind::TextBox`] | `TextBox` |
//! | `<Check>` | [`PlanKind::CheckBox`] | `CheckBox` |
//! | `<Progress>` | [`PlanKind::Progress`] (0..1) | `ProgressBar` (0..100) |
//! | `<Spinner>` | [`PlanKind::Spinner`] | `ProgressRing` |
//! | `<Divider>` | [`PlanKind::Divider`] | `Border` (1px 위 테두리) |
//! | `<Banner kind>` | [`PlanKind::Banner`] | `InfoBar` (severity) |
//! | `<Modal>` | [`PlanKind::Modal`] | `ContentDialog` |
//! | `<Raw>` | [`PlanKind::Raw`] | 클로저가 돌려준 뷰 |
//! | `<>…</>` | [`PlanKind::Fragment`] | `View::fragment` / `keyed_fragment` |
//!
//! ## 문서화된 한계
//!
//! - **자식 위치의 `<>`는 매크로가 펼친다**: `view!` 본문에서
//!   `<Col> <> … </> </Col>`처럼 쓰면 트리에 Fragment 노드가 남지 않는다(자식이 그대로
//!   이어 붙는다 — 순서는 보존된다). [`PlanKind::Fragment`]는 본문 **전체**가 `<>`일 때
//!   나온다. 계획은 두 경우 모두 올바르게 처리한다(`tests/plan.rs`).
//! - **키**: elm 트리는 key를 노출하지 않는다(코어가 아레나 슬롯 경로로 관리한다).
//!   자식 목록은 WinUI 층에서 **위치 기반 키**로 넘어간다 — Reactor의
//!   `keyed_children`을 쓰되 키는 인덱스다.
//! - **`on_enter`**: Reactor 0.100의 `TextBox`에는 키 이벤트가 없다. Enter는
//!   WinUI **가속기**(`KeyAccelerators`)로만 잡히므로, 계획에는
//!   [`PlanNode::enter`]로 남기고 WinUI 층이 `AcceleratorKey::Enter` 가속기를 붙인다
//!   (가속기를 받는 컨트롤이 `Grid`/`Button`뿐이라 `Grid`로 한 겹 감싼다 — 예제 05).
//! - **`<Tab>`**: Reactor의 `TabView`는 **컨테이너**라 형제 `<Tab>`들을 그대로
//!   옮길 수 없다. 그래서 버튼으로 옮기고, 진짜 `TabView`가 필요하면 `<Raw>`를 쓴다.

use std::any::Any;
use std::rc::Rc;

use elm_magic::widget::{
    BannerEl, ButtonEl, CheckEl, ColEl, DividerEl, FragmentEl, InputEl, ModalEl, ProgressEl, RawEl,
    RowEl, SpinnerEl, StrongEl, TabEl, TdEl, TextAreaEl, TextEl, ThEl,
};
use elm_magic::{Arena, Element};

/// 클릭 핸들러 — 코어의 `element::Handler`와 **같은 타입**이다
/// (그 별칭은 공개되지 않아 어댑터가 다시 선언한다).
pub type ClickHandler = Rc<dyn Fn(&mut Arena)>;
/// 값(문자열) 변경 핸들러 — `Input`/`TextArea`.
pub type ValueHandler = Rc<dyn Fn(&mut Arena, String)>;
/// bool 변경 핸들러 — `Check`.
pub type ToggleHandler = Rc<dyn Fn(&mut Arena, bool)>;
/// `<Raw>` 클로저 — WinUI 층은 `&mut Option<windows_reactor::View>`를 넣어 준다.
pub type RawFn = Rc<dyn Fn(&mut dyn Any)>;

/// `<Banner kind="…">`의 심각도 → `InfoBarSeverity`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// `kind="info"` (모르는 값도 여기로 온다).
    Info,
    /// `kind="success"` / `"ok"`.
    Success,
    /// `kind="warn"` / `"warning"`.
    Warning,
    /// `kind="error"`.
    Error,
}

impl Severity {
    /// `kind` 문자열 → 심각도 (모르는 값은 [`Severity::Info`]).
    pub fn from_kind(kind: &str) -> Severity {
        match kind {
            "error" => Severity::Error,
            "warn" | "warning" => Severity::Warning,
            "success" | "ok" => Severity::Success,
            _ => Severity::Info,
        }
    }
}

/// 계획 노드의 종류 = **WinUI 컨트롤의 종류**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanKind {
    /// `<Col>`(`vertical: true`) / `<Row>` — `StackPanel`.
    Stack {
        /// 세로 방향이면 `true`.
        vertical: bool,
    },
    /// `<Text>` / `<Strong>` — `TextBlock`.
    Text {
        /// `<Strong>`이면 굵게.
        strong: bool,
    },
    /// `<Button>` / `<Tab>` / 클릭 가능한 `<Th>` — `Button`.
    Button,
    /// `<Input>` / `<TextArea>` — `TextBox`.
    TextBox {
        /// `<TextArea>`면 여러 줄.
        multiline: bool,
    },
    /// `<Check>` — `CheckBox`.
    CheckBox,
    /// `<Progress>` — `ProgressBar` (계획은 0..1, WinUI는 0..100).
    Progress,
    /// `<Spinner>` — `ProgressRing`.
    Spinner,
    /// `<Divider>` — `Border`(1px 위 테두리).
    Divider,
    /// `<Banner kind="…">` — `InfoBar`.
    Banner {
        /// 종류별 심각도.
        severity: Severity,
    },
    /// `<Modal>` — `ContentDialog`.
    Modal,
    /// `<Raw>` — 클로저가 컨트롤을 돌려준다.
    Raw,
    /// `<>…</>` — 레이아웃 없는 묶음.
    Fragment,
}

impl PlanKind {
    /// 이 노드가 낳을 WinUI 컨트롤 이름 — [`Pass`]와 진단에 쓰인다.
    pub fn control(&self) -> &'static str {
        match self {
            PlanKind::Stack { .. } => "StackPanel",
            PlanKind::Text { .. } => "TextBlock",
            PlanKind::Button => "Button",
            PlanKind::TextBox { .. } => "TextBox",
            PlanKind::CheckBox => "CheckBox",
            PlanKind::Progress => "ProgressBar",
            PlanKind::Spinner => "ProgressRing",
            PlanKind::Divider => "Border",
            PlanKind::Banner { .. } => "InfoBar",
            PlanKind::Modal => "ContentDialog",
            PlanKind::Raw => "Raw",
            PlanKind::Fragment => "Fragment",
        }
    }
}

/// 노드에 붙는 이벤트 — WinUI 층이 `context.callback(..)`으로 감싸 메시지로 만든다.
#[derive(Clone)]
pub enum PlanEvent {
    /// 버튼/탭/헤더/컨테이너 클릭.
    Click(ClickHandler),
    /// 텍스트 변경(`Input`/`TextArea`).
    Change(ValueHandler),
    /// 체크 토글(`Check`).
    Toggle(ToggleHandler),
    /// 다이얼로그가 닫힘(`Modal`).
    Close(ClickHandler),
}

/// 계획 노드 — 플랫폼 독립 중간 표현.
///
/// 필드는 **그 위젯에만 의미가 있는 것만** 채워진다(예: `fraction`은 `Progress`만).
#[derive(Clone)]
pub struct PlanNode {
    /// 컨트롤 종류.
    pub kind: PlanKind,
    /// 표시 텍스트(라벨/본문/제목). 없으면 빈 문자열.
    pub text: String,
    /// `TextBox`의 현재 값.
    pub value: Option<String>,
    /// `CheckBox`의 체크 상태.
    pub checked: Option<bool>,
    /// `<Tab active={..}>`의 활성 여부.
    pub active: bool,
    /// `Progress`의 진행률 (0.0~1.0으로 클램프됨).
    pub fraction: Option<f64>,
    /// `disabled` 속성 — WinUI의 `is_enabled(false)`.
    pub disabled: bool,
    /// 주 이벤트(클릭/변경/토글/닫힘).
    pub event: Option<PlanEvent>,
    /// `Input`/`TextArea`의 Enter 핸들러 — WinUI 층이 `AcceleratorKey::Enter`
    /// **가속기**로 붙인다(`TextBox`에는 키 이벤트가 없다).
    ///
    /// 가속기 콜백은 인자가 없으므로 WinUI 층이 **그 프레임의 값**(`value`)을
    /// 캡처해서 넘긴다 — 핸들러가 받는 텍스트는 화면이 마지막으로 그린 값이다.
    pub enter: Option<ValueHandler>,
    /// `<Raw>` 클로저.
    pub raw: Option<RawFn>,
    /// 자식 (컨테이너/프래그먼트/모달만).
    pub children: Vec<PlanNode>,
}

impl PlanNode {
    fn new(kind: PlanKind, pass: &mut Pass) -> Self {
        pass.controls.push(kind.control());
        Self {
            kind,
            text: String::new(),
            value: None,
            checked: None,
            active: false,
            fraction: None,
            disabled: false,
            event: None,
            enter: None,
            raw: None,
            children: Vec::new(),
        }
    }

    /// 이 노드가 낳을 WinUI 컨트롤 이름.
    pub fn control(&self) -> &'static str {
        self.kind.control()
    }

    /// 컨테이너인가(자식을 갖는가).
    pub fn is_container(&self) -> bool {
        matches!(
            self.kind,
            PlanKind::Stack { .. } | PlanKind::Modal | PlanKind::Fragment
        )
    }
}

/// 한 프레임이 만든 계획의 요약 — 어댑터 검증용(egui `Pass`와 같은 계약).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pass {
    /// 만들 컨트롤 이름 (전위 순회 순서).
    pub controls: Vec<&'static str>,
    /// 버튼·탭·헤더·체크박스의 라벨 (순서대로).
    pub labeled: Vec<String>,
    /// 텍스트 노드들 (순서대로).
    pub texts: Vec<String>,
    /// 입력 필드 수.
    pub inputs: usize,
    /// 다이얼로그 수.
    pub dialogs: usize,
}

impl Pass {
    /// 특정 컨트롤이 몇 개 만들어지는가 — 회귀 테스트에 쓴다.
    pub fn count(&self, control: &str) -> usize {
        self.controls.iter().filter(|c| **c == control).count()
    }

    /// 이 라벨이 있었는가.
    pub fn has_label(&self, label: &str) -> bool {
        self.labeled.iter().any(|l| l == label)
    }

    /// 이 텍스트가 있었는가.
    pub fn has_text(&self, text: &str) -> bool {
        self.texts.iter().any(|t| t == text)
    }
}

/// 트리 → 계획 + 요약.
///
/// 한 프레임에 한 번 부른다. 순수 함수라 어디서든(테스트 포함) 돌릴 수 있다.
pub fn plan(root: &Element) -> (PlanNode, Pass) {
    let mut pass = Pass::default();
    let node = walk(root, &mut pass);
    (node, pass)
}

fn walk(el: &Element, pass: &mut Pass) -> PlanNode {
    match el {
        Element::Text(TextEl { text, .. }) => text_node(text, false, pass),
        Element::Strong(StrongEl { text, .. }) => text_node(text, true, pass),
        Element::Td(TdEl { text, .. }) => text_node(text, false, pass),
        Element::Col(ColEl {
            children, on_click, ..
        }) => stack(true, children, on_click.clone(), pass),
        Element::Row(RowEl {
            children, on_click, ..
        }) => stack(false, children, on_click.clone(), pass),
        Element::Button(ButtonEl {
            text,
            disabled,
            on_click,
            ..
        }) => {
            let mut n = PlanNode::new(PlanKind::Button, pass);
            n.text = text.clone();
            n.disabled = *disabled;
            n.event = on_click.clone().map(PlanEvent::Click);
            pass.labeled.push(text.clone());
            n
        }
        Element::Tab(TabEl {
            text,
            active,
            on_click,
            ..
        }) => {
            let mut n = PlanNode::new(PlanKind::Button, pass);
            n.text = text.clone();
            n.active = *active;
            n.event = on_click.clone().map(PlanEvent::Click);
            pass.labeled.push(text.clone());
            n
        }
        Element::Th(ThEl { text, on_click, .. }) => match on_click.clone() {
            // 정렬 가능한 헤더 → 클릭 가능한 버튼.
            Some(h) => {
                let mut n = PlanNode::new(PlanKind::Button, pass);
                n.text = text.clone();
                n.event = Some(PlanEvent::Click(h));
                pass.labeled.push(text.clone());
                n
            }
            // 정적 헤더 → 굵은 텍스트.
            None => text_node(text, true, pass),
        },
        Element::Input(InputEl {
            value,
            on_change,
            on_enter,
            ..
        }) => text_box(false, value, on_change.clone(), on_enter.clone(), pass),
        Element::TextArea(TextAreaEl {
            value,
            on_change,
            on_enter,
            ..
        }) => text_box(true, value, on_change.clone(), on_enter.clone(), pass),
        Element::Check(CheckEl {
            checked,
            label,
            on_change,
            ..
        }) => {
            let mut n = PlanNode::new(PlanKind::CheckBox, pass);
            n.text = label.clone();
            n.checked = Some(*checked);
            n.event = on_change.clone().map(PlanEvent::Toggle);
            pass.labeled.push(label.clone());
            n
        }
        Element::Banner(BannerEl { kind, text, .. }) => {
            let mut n = PlanNode::new(
                PlanKind::Banner {
                    severity: Severity::from_kind(kind),
                },
                pass,
            );
            n.text = text.clone();
            pass.texts.push(text.clone());
            n
        }
        Element::Spinner(SpinnerEl { .. }) => PlanNode::new(PlanKind::Spinner, pass),
        Element::Divider(DividerEl { .. }) => PlanNode::new(PlanKind::Divider, pass),
        Element::Progress(ProgressEl { value, .. }) => {
            let mut n = PlanNode::new(PlanKind::Progress, pass);
            // elm은 0..1 — WinUI 층이 0..100으로 바꾼다.
            n.fraction = Some(value.clamp(0.0, 1.0));
            n
        }
        Element::Modal(ModalEl {
            title,
            on_close,
            children,
            ..
        }) => {
            let mut n = PlanNode::new(PlanKind::Modal, pass);
            n.text = title.clone();
            n.event = on_close.clone().map(PlanEvent::Close);
            pass.dialogs += 1;
            n.children = children.iter().map(|c| walk(c, pass)).collect();
            n
        }
        Element::Raw(RawEl { widget, .. }) => {
            let mut n = PlanNode::new(PlanKind::Raw, pass);
            n.raw = Some(widget.clone());
            n
        }
        Element::Fragment(FragmentEl { children }) => {
            let mut n = PlanNode::new(PlanKind::Fragment, pass);
            n.children = children.iter().map(|c| walk(c, pass)).collect();
            n
        }
    }
}

fn text_node(text: &str, strong: bool, pass: &mut Pass) -> PlanNode {
    let mut n = PlanNode::new(PlanKind::Text { strong }, pass);
    n.text = text.to_string();
    pass.texts.push(text.to_string());
    n
}

fn stack(
    vertical: bool,
    children: &[Element],
    on_click: Option<ClickHandler>,
    pass: &mut Pass,
) -> PlanNode {
    let mut n = PlanNode::new(PlanKind::Stack { vertical }, pass);
    n.event = on_click.map(PlanEvent::Click);
    n.children = children.iter().map(|c| walk(c, pass)).collect();
    n
}

fn text_box(
    multiline: bool,
    value: &str,
    on_change: Option<ValueHandler>,
    on_enter: Option<ValueHandler>,
    pass: &mut Pass,
) -> PlanNode {
    let mut n = PlanNode::new(PlanKind::TextBox { multiline }, pass);
    n.value = Some(value.to_string());
    n.event = on_change.map(PlanEvent::Change);
    n.enter = on_enter;
    pass.inputs += 1;
    n
}
