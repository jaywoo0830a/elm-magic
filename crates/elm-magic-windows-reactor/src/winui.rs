//! WinUI 층 (**Windows 전용**) — 계획을 windows-reactor 컨트롤로 바꾼다.
//!
//! 이 파일은 `cfg(windows)`에서만 컴파일된다. 여기서 하는 일은 세 가지뿐이다.
//!
//! 1. [`ElmView`]가 `windows_reactor::Component`를 구현한다 — elm 프레임을 돌리고
//!    ([`elm_magic::frame`]) 계획을 만든 뒤([`crate::plan`]) 컨트롤로 옮긴다.
//! 2. elm 이벤트(`on_click`/`on_change`/`on_change(Check)`/`on_close`)를
//!    [`ElmMessage`]로 감싸 Reactor 메시지 큐에 넣는다 — `Component::update`에서
//!    그 핸들러를 아레나에 대해 실행한다.
//! 3. 스타일은 **옮기지 않는다**. WinUI 테마/리소스가 담당한다.
//!
//! ## 왜 `RefCell`인가
//!
//! Reactor의 `Component::view(&self, ..)`는 **불변**인데 elm의
//! [`elm_magic::frame`]은 `&mut Ctx`가 필요하다. 그래서 아레나를 `RefCell`로
//! 감싼다(`ElmView::ctx_mut`가 `RefMut`을 돌려준다). 프레임이 끝나면 빌림은
//! 즉시 풀리므로, 핸들러가 프레임 중에 다시 들어오는 경로는 없다.
//!
//! ## 효과/스트림 구동
//!
//! elm의 `<-`(효과)와 `->`(스트림)는 **런타임이 자동으로 돌리지 않는다**
//! (테스트에서는 `flush`/`pump`가 대신한다). Reactor에서는 `update`가 끝날 때
//! [`drive`]를 불러 **같은 발행(publish) 안에서** 결과를 반영한다 — 그래야
//! 화면이 한 번에 갱신된다. 지연 효과(`after`)는 `ctx.arena.set_now(..)`로 시계를
//! 밀어야 due가 되므로, 실무에서는 `ComponentContext::set_timeout`으로 메시지를
//! 보내 그 시점에 시계를 갱신하는 편이 맞다(예제 11 참고).

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use elm_magic::Ctx;
use windows_reactor::{
    Border, Button, CheckBox, ChildrenControl, Component, ComponentContext, ContentControl,
    ContentDialog, ContentDialogResult, FontWeight, InfoBar, InfoBarSeverity, KeyedView,
    Orientation, PointerEventInfo, ProgressBar, ProgressRing, StackPanel, TextBlock, TextBox,
    TextWrapping, Thickness, View, ViewContext,
};

use crate::plan::{plan, Pass, PlanEvent, PlanKind, PlanNode, Severity};

/// `<Raw>`가 값을 채우는 슬롯.
///
/// 이 어댑터에서 `<Raw>`는 **WinUI 컨트롤을 돌려준다** — gpui 어댑터와 달리
/// 반환값이 버려지지 않는다:
///
/// ```ignore
/// use elm_magic_windows_reactor::RawSlot;
/// use windows_reactor::{ScrollViewer, StackPanel, ChildrenControl};
///
/// <Raw>|out: &mut RawSlot| {
///     *out = Some(ScrollViewer::new().content(StackPanel::new()));
/// }</Raw>
/// ```
pub type RawSlot = Option<View>;

/// elm 핸들러를 Reactor 메시지 큐로 나르는 봉투.
///
/// 핸들러는 `Rc<dyn Fn(..)>`라 `'static`이기만 하면 되고, 아레나는 컴포넌트가
/// 소유하므로 [`Component::update`]에서 실행한다.
#[derive(Clone)]
pub enum ElmMessage {
    /// 클릭 — `Button`/`Border` 포인터 이벤트.
    Click(Rc<dyn Fn(&mut elm_magic::Arena)>),
    /// 텍스트 변경 — `TextBox::on_text_changed`.
    Change(Rc<dyn Fn(&mut elm_magic::Arena, String)>, String),
    /// 체크 토글 — `CheckBox::on_is_checked_changed`.
    Toggle(Rc<dyn Fn(&mut elm_magic::Arena, bool)>, bool),
    /// 다이얼로그 닫힘 — `ContentDialog::on_closed`.
    Close(Rc<dyn Fn(&mut elm_magic::Arena)>),
}

/// Reactor `Input` 요구사항(`Clone + PartialEq + 'static`)을 만족시키는 props 래퍼.
///
/// elm-magic의 `Component::Props`는 `Default`만 보장한다(생성된 `…Props`는
/// `Clone`만 파생한다). 그래서 **값 비교 대신 동일성**(같은 `Rc`)으로 비교한다 —
/// 부모가 새 props를 만들어 넘기면 `input_changed`가 불리고, 같은 값을 다시
/// 넘기면(`Rc` 복제) 불리지 않는다.
pub struct ElmInput<P> {
    props: Rc<P>,
}

impl<P> ElmInput<P> {
    /// props를 감싼다.
    pub fn new(props: P) -> Self {
        Self {
            props: Rc::new(props),
        }
    }

    /// 안쪽 props.
    pub fn props(&self) -> &P {
        &self.props
    }
}

impl<P> Clone for ElmInput<P> {
    fn clone(&self) -> Self {
        Self {
            props: Rc::clone(&self.props),
        }
    }
}

impl<P> PartialEq for ElmInput<P> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.props, &other.props)
    }
}

/// 값이 아니라 **동일성**(가리키는 `Rc`의 주소)을 보여준다.
///
/// `P`에 `Debug`를 요구하지 않으므로 props 타입이 `Debug`를 파생했는지와 무관하게
/// `assert_eq!`/`assert_ne!`/`{:?}`를 그대로 쓸 수 있다. 출력되는 주소가 같으면
/// 같은 props이며 `input_changed`가 불리지 않는다는 뜻이다.
impl<P> std::fmt::Debug for ElmInput<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ElmInput")
            .field(&Rc::as_ptr(&self.props))
            .finish()
    }
}

/// elm-magic 컴포넌트(`C`)를 Reactor 컴포넌트로 그리는 뷰.
///
/// `App::run_component::<ElmView<C>>(ElmInput::new(props))`로 창 하나를 띄운다.
/// 상태 아레나는 이 뷰가 소유하고, 이벤트는 [`ElmMessage`]로 돌아온다.
pub struct ElmView<C: elm_magic::Component + 'static> {
    props: ElmInput<C::Props>,
    /// 상태 아레나 — `view(&self)`에서도 프레임을 돌려야 해서 `RefCell`이다.
    ctx: RefCell<Ctx>,
    /// 마지막 프레임이 만든 계획 요약.
    pass: RefCell<Pass>,
}

impl<C: elm_magic::Component + 'static> ElmView<C> {
    /// props로 만든다 (`Component::create`가 부르는 경로).
    pub fn with_props(props: C::Props) -> Self {
        Self {
            props: ElmInput::new(props),
            ctx: RefCell::new(Ctx::new()),
            pass: RefCell::new(Pass::default()),
        }
    }

    /// 마지막 프레임이 만든 계획 요약 (어댑터 검증/진단용).
    pub fn pass(&self) -> Pass {
        self.pass.borrow().clone()
    }

    /// 상태 아레나 (읽기).
    pub fn ctx(&self) -> Ref<'_, Ctx> {
        self.ctx.borrow()
    }

    /// 상태 아레나 (쓰기) — 효과/스트림을 직접 구동할 때 쓴다.
    pub fn ctx_mut(&self) -> RefMut<'_, Ctx> {
        self.ctx.borrow_mut()
    }
}

impl<C: elm_magic::Component + 'static> Component for ElmView<C> {
    type Input = ElmInput<C::Props>;
    type Message = ElmMessage;

    fn create(input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        Self {
            // `Rc`를 그대로 물려받는다 — props 복제가 필요 없다(`C::Props`는 Clone이 아니다).
            props: input.clone(),
            ctx: RefCell::new(Ctx::new()),
            pass: RefCell::new(Pass::default()),
        }
    }

    fn input_changed(&mut self, input: &Self::Input, _context: &ComponentContext<Self>) {
        // 부모가 **새 props**를 만들었을 때만 온다(동일성 비교 — `ElmInput` 참고).
        self.props = input.clone();
    }

    fn update(&mut self, message: Self::Message, _context: &ComponentContext<Self>) {
        let mut ctx = self.ctx.borrow_mut();
        match message {
            ElmMessage::Click(handler) => handler(&mut ctx.arena),
            ElmMessage::Change(handler, value) => handler(&mut ctx.arena, value),
            ElmMessage::Toggle(handler, value) => handler(&mut ctx.arena, value),
            ElmMessage::Close(handler) => handler(&mut ctx.arena),
        }
        // elm의 `<-`(효과)와 `->`(스트림)는 런타임이 자동으로 돌리지 않는다.
        // Reactor에서는 **이 발행(publish) 안에서** 구동해 결과를 한 번에 반영한다.
        crate::drive::run(&mut ctx);
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        // 1) elm 프레임: 상태 → Element 트리 (`view(&self)`인데 아레나는 RefCell이다)
        let tree = {
            let mut ctx = self.ctx.borrow_mut();
            elm_magic::frame::<C>(&mut ctx, self.props.props())
        };
        // 2) 트리 → 계획 (플랫폼 독립 매핑)
        let (node, pass) = plan(&tree);
        *self.pass.borrow_mut() = pass;
        // 3) 계획 → WinUI 컨트롤
        build(&node, context)
    }
}

// ── 계획 → WinUI 컨트롤 ─────────────────────────────────────

/// 계획 노드 하나를 Reactor 뷰로 바꾼다 (자식은 재귀).
fn build<C: elm_magic::Component + 'static>(
    node: &PlanNode,
    context: &ViewContext<ElmView<C>>,
) -> View {
    match &node.kind {
        PlanKind::Stack { vertical } => {
            let panel = StackPanel::new().orientation(if *vertical {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            });
            let children = children_of(node, context);
            let view = if children.is_empty() {
                panel.children(())
            } else {
                panel.keyed_children(positional(children))
            };
            // 컨테이너 클릭: WinUI `StackPanel`에는 클릭 이벤트가 없어 `Border`로 받는다.
            // (배경 브러시가 없으면 여백은 히트 테스트되지 않는다 — WinUI 규칙.)
            match &node.event {
                Some(PlanEvent::Click(handler)) => {
                    let handler = Rc::clone(handler);
                    Border::new()
                        .on_pointer_released(context.callback(move |_: PointerEventInfo| {
                            ElmMessage::Click(Rc::clone(&handler))
                        }))
                        .content(view)
                }
                _ => view,
            }
        }
        PlanKind::Text { strong } => {
            let mut block = TextBlock::new().text(node.text.clone());
            if *strong {
                block = block.font_weight(FontWeight::BOLD);
            }
            block.into()
        }
        PlanKind::Button => {
            let mut button = Button::new();
            if node.disabled {
                button = button.is_enabled(false);
            }
            if let Some(PlanEvent::Click(handler)) = &node.event {
                let handler = Rc::clone(handler);
                button = button.on_click(
                    context.callback(move |_: ()| ElmMessage::Click(Rc::clone(&handler))),
                );
            }
            button.content(node.text.clone())
        }
        PlanKind::TextBox { multiline } => {
            let mut input = TextBox::new().text(node.value.clone().unwrap_or_default());
            if *multiline {
                // `<TextArea>` — Enter가 개행이 되도록 되돌림을 허용한다.
                input = input.accepts_return(true).text_wrapping(TextWrapping::Wrap);
            }
            if let Some(PlanEvent::Change(handler)) = &node.event {
                let handler = Rc::clone(handler);
                input =
                    input.on_text_changed(context.callback(move |value: String| {
                        ElmMessage::Change(Rc::clone(&handler), value)
                    }));
            }
            input.into()
        }
        PlanKind::CheckBox => {
            let mut check = CheckBox::new().is_checked(node.checked.unwrap_or(false));
            if let Some(PlanEvent::Toggle(handler)) = &node.event {
                let handler = Rc::clone(handler);
                check =
                    check.on_is_checked_changed(context.callback(move |value: bool| {
                        ElmMessage::Toggle(Rc::clone(&handler), value)
                    }));
            }
            check.content(node.text.clone())
        }
        PlanKind::Progress => ProgressBar::new()
            .minimum(0.0)
            .maximum(100.0)
            // elm은 0..1, WinUI는 0..100.
            .value(node.fraction.unwrap_or(0.0) * 100.0)
            .into(),
        PlanKind::Spinner => ProgressRing::new().is_active(true).into(),
        PlanKind::Divider => Border::new()
            .border_thickness(Thickness::new(0.0, 1.0, 0.0, 0.0))
            .into(),
        other => build_tail(node, other, context),
    }
}

/// 나머지 종류 — 배너/모달/`<Raw>`/프래그먼트.
fn build_tail<C: elm_magic::Component + 'static>(
    node: &PlanNode,
    kind: &PlanKind,
    context: &ViewContext<ElmView<C>>,
) -> View {
    match kind {
        PlanKind::Banner { severity } => InfoBar::new()
            .severity(severity_of(*severity))
            .message(node.text.clone())
            .is_open(true)
            .into(),
        PlanKind::Modal => {
            // `<Modal>`은 WinUI `ContentDialog`로 옮긴다. `on_close`를 등록하지 않으면
            // 사용자가 닫아도 elm 상태가 그대로라 다음 프레임에 다시 열린다 — 예제 19 참고.
            let mut dialog = ContentDialog::new().title(node.text.clone()).is_open(true);
            if let Some(PlanEvent::Close(handler)) = &node.event {
                let handler = Rc::clone(handler);
                dialog = dialog.on_closed(context.callback(move |_: ContentDialogResult| {
                    ElmMessage::Close(Rc::clone(&handler))
                }));
            }
            let body = children_of(node, context);
            let content = if body.is_empty() {
                View::empty()
            } else {
                View::keyed_fragment(positional(body))
            };
            dialog.content(content)
        }
        PlanKind::Raw => {
            // 이 어댑터의 `<Raw>`는 **뷰를 돌려준다** (gpui 어댑터와 다른 점).
            let mut slot: RawSlot = None;
            if let Some(widget) = &node.raw {
                widget(&mut slot as &mut dyn Any);
            }
            slot.unwrap_or_else(View::empty)
        }
        PlanKind::Fragment => {
            let children = children_of(node, context);
            if children.is_empty() {
                View::empty()
            } else {
                View::keyed_fragment(positional(children))
            }
        }
        // 위에서 모두 처리한 종류 — `build`가 여기로 보내지 않는다.
        _ => View::empty(),
    }
}

fn children_of<C: elm_magic::Component + 'static>(
    node: &PlanNode,
    context: &ViewContext<ElmView<C>>,
) -> Vec<View> {
    node.children
        .iter()
        .map(|child| build(child, context))
        .collect()
}

/// 자식 목록에 붙일 키 — elm 트리는 key를 노출하지 않으므로 **위치**가 키다.
///
/// elm 코어는 keyed 슬롯의 상태를 아레나에서 관리하지만 그 키는 트리에 실리지
/// 않는다. 그래서 Reactor 쪽 식별은 "같은 위치 = 같은 컨트롤"이 된다(문서화된 한계).
fn positional(children: Vec<View>) -> impl Iterator<Item = KeyedView> {
    children
        .into_iter()
        .enumerate()
        .map(|(index, view)| KeyedView::new(index as u64, view))
}

fn severity_of(severity: Severity) -> InfoBarSeverity {
    match severity {
        Severity::Info => InfoBarSeverity::Informational,
        Severity::Success => InfoBarSeverity::Success,
        Severity::Warning => InfoBarSeverity::Warning,
        Severity::Error => InfoBarSeverity::Error,
    }
}
