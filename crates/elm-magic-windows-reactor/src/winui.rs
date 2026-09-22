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
//!
//! ## 업스트림이 `view()`에서 하는 일을 누가 하나 (호환성)
//!
//! 업스트림 샘플은 **컴포넌트의 `view()`에서** 창을 선언하고(`window_title`,
//! `window_visuals`, `on_window_size`) 가속기를 붙인다(`KeyAccelerators`).
//! elm 코드는 `ViewContext`를 볼 수 없으므로, 그 자리를 [`ElmInput`]이 대신한다:
//!
//! | 업스트림 (샘플) | 이 어댑터 |
//! |---|---|
//! | `context.window_title("..")` | `ElmInput::new(props).window_title("..")` |
//! | `context.window_visuals(WindowVisuals::new()..)` | `.window_visuals(visuals)` |
//! | `context.on_window_size(context.callback(Msg::Resized))` | `.on_window_size(\|size\| ..)` |
//! | `context.on_color_scheme(..)` | `.on_color_scheme(\|scheme\| ..)` |
//! | `Grid::new().key_accelerators(..)`를 루트에 | elm `on_key("Ctrl+R")` → 자동 매핑 |
//! | `Grid::new().key_accelerators(Enter)`로 입력 제출 | elm `<Input on_enter={..}>` → 자동 매핑 |
//! | `App::run_component::<C>(())` | `App::run_component::<ElmView<C>>(ElmInput::new(props))` |
//!
//! 가속기는 `AcceleratorKey`(0.100.0)가 아는 키만 매핑된다 — `R` · `Enter` ·
//! 사칙연산 · 숫자패드 + `Ctrl`. 그 밖의 키(`Ctrl+S` 등)는 **조용히 건너뛴다**
//! (호스트가 직접 `KeyAccelerators`를 붙이거나 `<Raw>`를 쓰면 된다).
//! 가속기를 받을 수 있는 컨트롤은 `Grid`/`Button`뿐이라, 매핑이 필요하면
//! 루트를 `Grid`로 한 겹 감싼다(레이아웃은 그대로다 — spacing 0).

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use elm_magic::Ctx;
use windows_reactor::{
    AcceleratorKey, AcceleratorModifiers, Border, Button, CheckBox, ChildrenControl, ColorScheme,
    Component, ComponentContext, ContentControl, ContentDialog, ContentDialogResult, FontWeight,
    Grid, InfoBar, InfoBarSeverity, KeyAccelerator, KeyAccelerators, KeyedView, Orientation,
    PointerEventInfo, ProgressBar, ProgressRing, StackPanel, TextBlock, TextBox, TextWrapping,
    Thickness, View, ViewContext, WindowSize, WindowVisuals,
};

use crate::plan::{plan, Pass, PlanEvent, PlanKind, PlanNode, Severity, ValueHandler};

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
    /// 키보드 가속기 — elm `on_key(..)`/`on_enter`가 WinUI `KeyAccelerators`로 내려온 것.
    ///
    /// `KeyAccelerators`는 `Grid`/`Button`에만 붙일 수 있고 `AcceleratorKey`가 아는
    /// 키도 한정돼 있다 — 그래서 **문자열 키를 아레나 핸들러로 바꿔 나르는** 것이
    /// 이 변형의 일이다(`accelerator_for` 참고).
    Key(Rc<dyn Fn(&mut elm_magic::Arena)>),
}

/// 창 선언 — 업스트림이 `ViewContext`로 하는 일 중 **elm 코드가 쓸 수 있어야 하는 것**.
///
/// `ViewContext`는 컴포넌트의 `view()`에서만 손에 쥘 수 있는데, elm 코드는
/// 그 자리에 없다. 그래서 그 통로를 [`ElmInput`]의 빌더로 옮겼다.
#[derive(Clone)]
struct WindowDeclaration {
    title: Option<String>,
    visuals: Option<WindowVisuals>,
    on_window_size: Option<Rc<dyn Fn(WindowSize)>>,
    on_color_scheme: Option<Rc<dyn Fn(ColorScheme)>>,
    /// elm `on_key(..)`/`on_enter`를 WinUI 가속기로 내려보낼지 (기본 `true`).
    accelerators: bool,
}

/// Reactor `Input` 요구사항(`Clone + PartialEq + 'static`)을 만족시키는 props 래퍼.
///
/// elm-magic의 `Component::Props`는 `Default`만 보장한다(생성된 `…Props`는
/// `Clone`만 파생한다). 그래서 **값 비교 대신 동일성**(같은 `Rc`)으로 비교한다 —
/// 부모가 새 props를 만들어 넘기면 `input_changed`가 불리고, 같은 값을 다시
/// 넘기면(`Rc` 복제) 불리지 않는다.
///
/// ## 창/가속기 선언 (업스트림 호환)
///
/// 업스트림 샘플은 `view()`에서 창을 선언하고 가속기를 단다. elm 쪽은
/// `ViewContext`가 없으므로 **같은 선언을 여기서** 한다 — 선언은 매 발행
/// (`view()` 호출)마다 그대로 적용되므로 elm 상태에 따라 제목이 바뀌게 할 수도
/// 있다(그때는 부모가 매 프레임 새 `ElmInput`을 만든다 — 예제 02의 규칙).
///
/// ```ignore
/// App::run_component::<ElmView<Counter>>(
///     ElmInput::new(CounterProps::default())
///         .window_title("elm-magic — 카운터")
///         .window_visuals(WindowVisuals::new().client_size(360.0, 220.0)),
/// )
/// ```
pub struct ElmInput<P> {
    props: Rc<P>,
    window: WindowDeclaration,
}

impl Default for WindowDeclaration {
    fn default() -> Self {
        Self {
            title: None,
            visuals: None,
            on_window_size: None,
            on_color_scheme: None,
            // elm이 `on_key`/`on_enter`로 선언한 것은 **자동으로** 내려보낸다
            // (업스트림에서 선언이 곧 효과인 것과 같다).
            accelerators: true,
        }
    }
}

impl<P> ElmInput<P> {
    /// props를 감싼다.
    pub fn new(props: P) -> Self {
        Self {
            props: Rc::new(props),
            window: WindowDeclaration::default(),
        }
    }

    /// 안쪽 props.
    pub fn props(&self) -> &P {
        &self.props
    }

    /// 창 제목 — 업스트림의 `context.window_title(..)`.
    pub fn window_title(mut self, title: impl Into<String>) -> Self {
        self.window.title = Some(title.into());
        self
    }

    /// 창 크기/테마 — 업스트림의 `context.window_visuals(..)`.
    pub fn window_visuals(mut self, visuals: WindowVisuals) -> Self {
        self.window.visuals = Some(visuals);
        self
    }

    /// 창 크기 변화 관찰 — 업스트림의 `context.on_window_size(..)`.
    ///
    /// 호출은 **메시지로 큐에 들어간다**(Reactor 규칙) — 그래서 관찰자는
    /// `update`가 도는 시점에 불린다. 받은 값을 elm에 반영하려면 호스트가 그 값을
    /// 소유하고 props로 내려보낸다(예제 20).
    pub fn on_window_size(mut self, observe: impl Fn(WindowSize) + 'static) -> Self {
        self.window.on_window_size = Some(Rc::new(observe));
        self
    }

    /// 라이트/다크 전환 관찰 — 업스트림의 `context.on_color_scheme(..)`.
    pub fn on_color_scheme(mut self, observe: impl Fn(ColorScheme) + 'static) -> Self {
        self.window.on_color_scheme = Some(Rc::new(observe));
        self
    }

    /// elm `on_key`/`on_enter`를 WinUI 가속기로 내려보낼지 (기본 `true`).
    ///
    /// 끄면 elm의 키 선언은 **헤드리스 계약으로만** 남는다(호스트가 직접
    /// `KeyAccelerators`를 붙이는 경우 — 예제 05의 두 번째 예).
    pub fn accelerators(mut self, enabled: bool) -> Self {
        self.window.accelerators = enabled;
        self
    }
}

impl<P> Clone for ElmInput<P> {
    fn clone(&self) -> Self {
        Self {
            props: Rc::clone(&self.props),
            // 창 선언도 함께 물려받는다 — 선언은 매 발행마다 그대로 적용된다.
            window: self.window.clone(),
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
            ElmMessage::Key(handler) => handler(&mut ctx.arena),
        }
        // elm의 `<-`(효과)와 `->`(스트림)는 런타임이 자동으로 돌리지 않는다.
        // Reactor에서는 **이 발행(publish) 안에서** 구동해 결과를 한 번에 반영한다.
        crate::drive::run(&mut ctx);
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        // 0) 창 선언 — 업스트림이 `view()`에서 하는 일과 **같은 자리**다.
        //    elm 코드는 `ViewContext`를 볼 수 없으므로 `ElmInput`이 그 통로다.
        if let Some(title) = &self.props.window.title {
            context.window_title(title.clone());
        }
        if let Some(visuals) = self.props.window.visuals.clone() {
            context.window_visuals(visuals);
        }
        if let Some(observe) = &self.props.window.on_window_size {
            let observe = Rc::clone(observe);
            // 관찰자는 호스트 콜백이므로 메시지를 거치지 않는다(값만 전달).
            context.on_window_size(move |size: WindowSize| observe(size));
        }
        if let Some(observe) = &self.props.window.on_color_scheme {
            let observe = Rc::clone(observe);
            context.on_color_scheme(move |scheme: ColorScheme| observe(scheme));
        }
        // 1) elm 프레임: 상태 → Element 트리 (`view(&self)`인데 아레나는 RefCell이다)
        let tree = {
            let mut ctx = self.ctx.borrow_mut();
            elm_magic::frame::<C>(&mut ctx, self.props.props())
        };
        // 2) 트리 → 계획 (플랫폼 독립 매핑)
        let (node, pass) = plan(&tree);
        *self.pass.borrow_mut() = pass;
        // 3) 계획 → WinUI 컨트롤
        let root = build(&node, context);
        // 4) elm `on_key` → WinUI `KeyAccelerators` (업스트림 `calculator`의 자리)
        self.attach_key_accelerators(root, context)
    }
}

impl<C: elm_magic::Component + 'static> ElmView<C> {
    /// 마지막 프레임이 `on_key`로 등록한 핸들러를 WinUI 가속기로 내려보낸다.
    ///
    /// - 매핑할 수 있는 키가 하나도 없으면 **루트를 손대지 않는다**(불필요한 `Grid`
    ///   래퍼를 만들지 않는다).
    /// - 하나라도 있으면 루트를 `Grid`로 감싼다 — 가속기를 받는 컨트롤이
    ///   `Grid`/`Button`뿐이기 때문이다(`Spacing` 0이라 레이아웃은 그대로다).
    /// - `Ctrl+S`처럼 `AcceleratorKey`에 없는 키는 조용히 건너뛴다 — 그 키는
    ///   호스트가 직접 붙이거나 `<Raw>`에서 처리한다.
    /// - WinUI 가속기는 **그 컨트롤의 스코프에 포커스가 있을 때** 동작한다(업스트림
    ///   `calculator`도 루트 `Grid`에 달아 같은 성질을 갖는다) — 창 어디에서나 잡히는
    ///   전역 단축키가 필요하면 호스트가 붙이는 편이 맞다.
    fn attach_key_accelerators(&self, root: View, context: &ViewContext<Self>) -> View {
        if !self.props.window.accelerators {
            return root;
        }
        // `frame`이 아레나를 빌린 동안에는 읽을 수 없다 — 프레임 뒤에 읽는다
        // (`begin_frame`이 매 프레임 `keys`를 비우고 다시 채운다).
        let keys: Vec<(String, Rc<dyn Fn(&mut elm_magic::Arena)>)> = {
            let ctx = self.ctx.borrow();
            ctx.keys
                .iter()
                .map(|(key, handler)| (key.clone(), Rc::clone(handler)))
                .collect()
        };
        let accelerators: Vec<KeyAccelerator> = keys
            .iter()
            .filter_map(|(key, handler)| {
                let (accelerator, modifiers) = accelerator_for(key)?;
                let handler = Rc::clone(handler);
                Some(KeyAccelerator::new(
                    accelerator,
                    modifiers,
                    context.callback(move |_: ()| ElmMessage::Key(Rc::clone(&handler))),
                ))
            })
            .collect();
        if accelerators.is_empty() {
            return root;
        }
        Grid::new()
            .key_accelerators(KeyAccelerators::new(accelerators))
            .children([root])
            .into()
    }
}

/// elm 키 문자열(`on_key("Ctrl+R")`) → WinUI 가속기.
///
/// `AcceleratorKey`(0.100.0)가 아는 키는 `R` · `Enter` · 사칙연산 · 숫자패드뿐이고
/// 수정자도 `Control` 하나뿐이다(Shift/Alt 없음). 그래서 **아는 것만** 매핑하고
/// 나머지는 `None`이다 — 조용히 무시되므로, 동작하지 않는 키는 호스트가 붙여야 한다.
fn accelerator_for(key: &str) -> Option<(AcceleratorKey, AcceleratorModifiers)> {
    let mut rest = key.trim();
    let mut modifiers = AcceleratorModifiers::None;
    // `Ctrl+R` / `Control+R` (대소문자 무시). Shift/Alt는 표현할 수 없다.
    for prefix in ["Ctrl+", "Control+", "CTRL+"] {
        if rest.len() >= prefix.len() && rest[..prefix.len()].eq_ignore_ascii_case(prefix) {
            modifiers = AcceleratorModifiers::Control;
            rest = &rest[prefix.len()..];
            break;
        }
    }
    if rest.len() >= 6 && rest[..6].eq_ignore_ascii_case("Shift+") {
        return None;
    }
    let name = rest.to_ascii_lowercase();
    let accelerator = match name.as_str() {
        "enter" | "return" | "엔터" => AcceleratorKey::Enter,
        "r" => AcceleratorKey::R,
        "add" | "+" => AcceleratorKey::Add,
        "subtract" | "-" => AcceleratorKey::Subtract,
        "multiply" | "*" => AcceleratorKey::Multiply,
        "divide" | "/" => AcceleratorKey::Divide,
        "decimal" | "." => AcceleratorKey::Decimal,
        "numpad0" | "num0" | "numpad_0" => AcceleratorKey::NumberPad0,
        "numpad1" | "num1" | "numpad_1" => AcceleratorKey::NumberPad1,
        "numpad2" | "num2" | "numpad_2" => AcceleratorKey::NumberPad2,
        "numpad3" | "num3" | "numpad_3" => AcceleratorKey::NumberPad3,
        "numpad4" | "num4" | "numpad_4" => AcceleratorKey::NumberPad4,
        "numpad5" | "num5" | "numpad_5" => AcceleratorKey::NumberPad5,
        "numpad6" | "num6" | "numpad_6" => AcceleratorKey::NumberPad6,
        "numpad7" | "num7" | "numpad_7" => AcceleratorKey::NumberPad7,
        "numpad8" | "num8" | "numpad_8" => AcceleratorKey::NumberPad8,
        "numpad9" | "num9" | "numpad_9" => AcceleratorKey::NumberPad9,
        _ => return None,
    };
    Some((accelerator, modifiers))
}

// ── 계획 → WinUI 컨트롤 ─────────────────────────────────────

/// 계획 노드 하나를 Reactor 뷰로 바꾼다 (자식은 재귀).
///
/// 컨트롤을 만든 뒤 [`PlanNode::enter`]에 적힌 Enter 핸들러를 **가속기로** 붙인다 —
/// `TextBox`에는 키 이벤트가 없어서 Enter는 `KeyAccelerators`로만 잡힌다
/// (업스트림 `calculator`가 Enter를 가속기로 받는 것과 같은 방법).
fn build<C: elm_magic::Component + 'static>(
    node: &PlanNode,
    context: &ViewContext<ElmView<C>>,
) -> View {
    let view = build_control(node, context);
    match &node.enter {
        Some(handler) => with_enter_accelerator(view, node, handler, context),
        None => view,
    }
}

/// 계획 노드를 **컨트롤 하나로** 바꾼다 (종류별 매핑).
fn build_control<C: elm_magic::Component + 'static>(
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

/// elm `<Input on_enter={..}>`를 WinUI **Enter 가속기**로 옮긴다.
///
/// elm의 `on_enter`는 `(상태, 현재 텍스트)`를 받는 핸들러다 — 가속기의 콜백은
/// 인자가 없으므로 **그 프레임의 값**(`node.value`)을 캡처해 넘긴다. 화면이 다시
/// 그려질 때마다 새 클로저가 만들어지므로 캡처된 값은 항상 최신이다.
///
/// 가속기를 받는 컨트롤은 `Grid`/`Button`뿐이라 `Grid`로 한 겹 감싼다(간격 0).
fn with_enter_accelerator<C: elm_magic::Component + 'static>(
    view: View,
    node: &PlanNode,
    handler: &ValueHandler,
    context: &ViewContext<ElmView<C>>,
) -> View {
    let handler = Rc::clone(handler);
    let value = node.value.clone().unwrap_or_default();
    let callback = context.callback(move |_: ()| {
        let handler = Rc::clone(&handler);
        let value = value.clone();
        ElmMessage::Key(
            Rc::new(move |arena: &mut elm_magic::Arena| handler(arena, value.clone()))
                as Rc<dyn Fn(&mut elm_magic::Arena)>,
        )
    });
    let accelerators = KeyAccelerators::new([KeyAccelerator::new(
        AcceleratorKey::Enter,
        AcceleratorModifiers::None,
        callback,
    )]);
    Grid::new()
        .key_accelerators(accelerators)
        .children([view])
        .into()
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

#[cfg(test)]
mod tests {
    // WinUI 컨텍스트가 필요 없는 **순수 매핑**만 시험한다(창은 띄우지 않는다).
    use super::{accelerator_for, WindowDeclaration};
    use windows_reactor::{AcceleratorKey, AcceleratorModifiers};

    /// 업스트림이 쓰는 키는 매핑되고, 0.100.0에 없는 키는 **조용히** 빠진다.
    #[test]
    fn only_upstream_accelerator_keys_map() {
        assert_eq!(
            accelerator_for("Enter"),
            Some((AcceleratorKey::Enter, AcceleratorModifiers::None))
        );
        assert_eq!(
            accelerator_for("Ctrl+R"),
            Some((AcceleratorKey::R, AcceleratorModifiers::Control))
        );
        // 대소문자는 가리지 않는다.
        assert_eq!(
            accelerator_for("control+r"),
            Some((AcceleratorKey::R, AcceleratorModifiers::Control))
        );
        assert_eq!(
            accelerator_for("Numpad7"),
            Some((AcceleratorKey::NumberPad7, AcceleratorModifiers::None))
        );
        assert_eq!(
            accelerator_for("+"),
            Some((AcceleratorKey::Add, AcceleratorModifiers::None))
        );
        // 0.100.0의 `AcceleratorKey`/`AcceleratorModifiers`에 없는 것들 —
        // 이 키들은 호스트가 직접 붙이거나 `<Raw>`에서 처리해야 한다.
        assert_eq!(accelerator_for("Ctrl+S"), None);
        assert_eq!(accelerator_for("Shift+Enter"), None);
        assert_eq!(accelerator_for("F5"), None);
        assert_eq!(accelerator_for("Tab"), None);
    }

    /// 기본값은 "elm이 선언한 키를 그대로 내려보낸다"다 — 호스트가 직접 붙이는
    /// 경우에만 끈다(예제 05의 두 번째 예).
    #[test]
    fn accelerators_default_to_on() {
        assert!(WindowDeclaration::default().accelerators);
    }
}
