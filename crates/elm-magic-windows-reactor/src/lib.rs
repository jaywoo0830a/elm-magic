//! windows-reactor 어댑터 (사양서 7.2) — elm-magic을 **WinUI 3**로 그린다.
//!
//! [`windows-reactor`](https://crates.io/crates/windows-reactor)는 Microsoft
//! windows-rs의 선언적 WinUI 3 라이브러리다. 컴포넌트가 상태를 소유하고
//! `view()`가 컨트롤 트리를 돌려주는 모델이라 elm-magic과 **같은 모양**이다:
//!
//! ```text
//! elm-magic:  상태 → 이벤트(할당) → render() → Element 트리
//! reactor:    상태 → Message      → update()  → View(컨트롤 트리)
//! ```
//!
//! 그래서 어댑터는 두 모델 사이의 **번역기**다. elm 컴포넌트는 플랫폼을 모른 채
//! 그대로 재사용되고(`view!`/`ui!` 본문은 손대지 않는다), 이 크레이트가
//! `Element` 트리를 Reactor `View`로 옮긴다.
//!
//! ## 이 어댑터는 **스타일 계층을 지원하지 않는다** (의도)
//!
//! egui/gpui 어댑터는 코어가 해석한 `ResolvedStyle`을 테마 토큰으로 옮긴다.
//! 이 어댑터는 **`css!`도 `class`도 읽지 않는다**:
//!
//! - `Element::class`를 보지 않고, `style::resolve`/`resolved_style_in`을
//!   호출하지 않는다. `css!`로 쓴 규칙은 이 백엔드에서 **아무 효과가 없다**.
//! - 색·간격·정렬·글꼴은 **WinUI의 몫**이다(테마 리소스, `Border`,
//!   `FontWeight`, `ResourceOverrides`). Reactor는 이미 테마/리소스 계층을
//!   갖고 있고, elm-magic의 CSS 부분집합으로 그걸 다시 표현하면 두 계층이
//!   어긋난다.
//! - WinUI 수준 조정이 필요하면 [`RawSlot`]을 쓰는 `<Raw>`가 정식 통로다.
//!   다른 어댑터와 달리 **Reactor는 뷰를 돌려줄 수 있으므로** `<Raw>`가
//!   실제 컨트롤을 낳는다(gpui에서는 부수 효과 전용이었다).
//!
//! ## 구조 (두 층)
//!
//! | 층 | 파일 | 플랫폼 | 하는 일 |
//! |---|---|---|---|
//! | 계획 | [`mod@crate::plan`] | 어디서나 | `Element` → [`PlanNode`] + [`Pass`] (매핑 규칙, 테스트 대상) |
//! | WinUI | `winui` | Windows만 | [`PlanNode`] → `windows_reactor::View` + 이벤트 연결 |
//! | 구동 | [`drive`] | 어디서나 | 효과/스트림/키 핸들러 실행 (앱이 돌려야 하는 것들) |
//!
//! 계획 층이 플랫폼 독립이라 **리눅스 CI에서 매핑 규칙이 검증된다**
//! (`tests/plan.rs`). WinUI 층은 계획을 컨트롤로 바꾸는 얇은 코드만 갖는다.
//!
//! ## 효과/스트림은 앱이 구동한다
//!
//! elm-magic의 런타임은 렌더만 한다 — `<-`(효과), `->`(스트림), `on_key` 핸들러는
//! 호스트가 돌려야 한다(헤드리스 테스트의 `flush`/`pump`/`press_key`에 해당).
//! Reactor에서는 [`ElmView`]의 `update`가 **메시지 하나를 처리한 뒤 같은 발행
//! 안에서** [`drive::run`]을 부른다. 직접 구동할 때는
//! [`ElmView::ctx_mut`]와 [`drive`]를 쓴다.
//!
//! 키는 예외적으로 **어댑터가 대신 내려보낸다**: elm이 `on_key`/`on_enter`로
//! 선언한 것은 WinUI `KeyAccelerators`로 매핑되고, 키가 눌리면
//! [`ElmMessage::Key`]로 돌아와 `update`에서 실행된다(업스트림이 `view()`에서
//! 가속기를 다는 자리와 같다). 매핑할 수 없는 키만 호스트 몫으로 남는다.
//!
//! ## 업스트림 windows-rs 샘플과의 대응
//!
//! 업스트림 샘플([`crates/samples/reactor`](https://github.com/microsoft/windows-rs/tree/master/crates/samples))
//! 과 **같은 자리에서 같은 API**를 쓴다 — elm 코드에 `ViewContext`가 없어서
//! "컴포넌트의 `view()`가 선언하는 것"만 [`ElmInput`] 빌더로 옮겼다.
//!
//! | 업스트림 (`counter` · `window` · `calculator` …) | 이 어댑터 |
//! |---|---|
//! | `App::run_component::<C>(())` | `App::run_component::<ElmView<C>>(ElmInput::new(props))` |
//! | `App::run_windows([a, b])` | 같은 API (`ElmView`가 `Component`다) |
//! | `context.window_title("..")` | `ElmInput::new(props).window_title("..")` |
//! | `context.window_visuals(..)` | `.window_visuals(WindowVisuals::new()..)` |
//! | `context.on_window_size(..)` | `.on_window_size(\|size\| ..)` |
//! | `context.on_color_scheme(..)` | `.on_color_scheme(\|scheme\| ..)` |
//! | `context.open_window(view)` | 그대로 (예제 20) |
//! | `context.spawn_background(..)` | 호스트 컴포넌트에서 그대로 (예제 11 · 12) |
//! | `Grid::new().key_accelerators(..)` | elm `on_key("Ctrl+R")` → 자동 매핑 |
//! | `KeyAccelerators(Enter)`로 입력 제출 | elm `<Input on_enter={..}>` → 자동 매핑 |
//! | `ElementRef::new()` + `request_focus()` | 어댑터가 만든 컨트롤에는 참조가 없다 — `<Raw>`에서 직접 |
//! | `KeyedView::new(키, 뷰)` | 위치 기반 키 (elm 트리가 키를 노출하지 않는다) |
//! | `Context::new(..)` + `View::provide` | elm `#[store]` (예제 13) |
//!
//! ## 최소 사용법
//!
//! ```ignore
//! use elm_magic::prelude::*;
//! use elm_magic_windows_reactor::{ElmInput, ElmView};
//!
//! elm_magic::view! {
//!     fn Counter(n = 0) {
//!         <Col>
//!             "Count: {n}"
//!             <Button on_click={n += 1}>"+"</Button>
//!         </Col>
//!     }
//! }
//!
//! fn main() -> windows_core::Result<()> {
//!     windows_reactor::App::run_component::<ElmView<Counter>>(
//!         ElmInput::new(CounterProps::default()).window_title("elm-magic — 카운터"),
//!     )
//! }
//! ```
//!
//! ## 알아둘 것
//!
//! - **Windows 전용**: `windows-reactor`는 WASDK 런타임이 필요하다. 이 크레이트는
//!   `cfg(windows)`에서만 WinUI 코드를 컴파일하고, 다른 플랫폼에서는 계획 층만 남는다.
//! - **스타일 없음**: 위의 "스타일 계층을 지원하지 않는다" 참고.
//! - **키는 WinUI가 아는 것만**: elm `on_key`/`on_enter`는 `AcceleratorKey`(0.100.0)에
//!   있는 키(`R` · `Enter` · 사칙연산 · 숫자패드 + `Ctrl`)만 매핑된다. 그 밖의 키는
//!   조용히 건너뛰므로 호스트가 `KeyAccelerators`를 직접 붙이거나 `<Raw>`를 쓴다.
//!   `.accelerators(false)`로 자동 매핑을 끌 수도 있다.
//! - **창 선언은 루트에서**: `window_title`/`window_visuals`는 창에 하나뿐이므로
//!   루트 컴포넌트의 [`ElmInput`]에서 선언한다(업스트림도 같은 규칙이다).
//! - **키 없는 목록**: elm 트리는 key를 노출하지 않으므로(코어가 아레나에서
//!   관리한다) 자식 목록은 **위치 기반 키**로 넘어간다 — Reactor의
//!   `keyed_children`을 쓰되 키는 인덱스다(문서화된 한계).

#![warn(missing_docs)]

pub mod drive;
pub mod plan;

#[cfg(windows)]
mod winui;

#[cfg(windows)]
pub use winui::{ElmInput, ElmMessage, ElmView, RawSlot};

pub use plan::{
    plan, ClickHandler, Pass, PlanEvent, PlanKind, PlanNode, RawFn, Severity, ToggleHandler,
    ValueHandler,
};
