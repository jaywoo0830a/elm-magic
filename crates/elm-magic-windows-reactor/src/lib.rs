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
//! | 계획 | [`plan`] | 어디서나 | `Element` → [`PlanNode`] + [`Pass`] (매핑 규칙, 테스트 대상) |
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
//!     windows_reactor::App::run_component::<ElmView<Counter>>(ElmInput::new(CounterProps::default()))
//! }
//! ```
//!
//! ## 알아둘 것
//!
//! - **Windows 전용**: `windows-reactor`는 WASDK 런타임이 필요하다. 이 크레이트는
//!   `cfg(windows)`에서만 WinUI 코드를 컴파일하고, 다른 플랫폼에서는 계획 층만 남는다.
//! - **스타일 없음**: 위의 "스타일 계층을 지원하지 않는다" 참고.
//! - **`on_enter` 미지원**: Reactor 0.100의 `TextBox`에는 키 이벤트가 없어
//!   Enter는 라우티드 키 콜백(`Border`)으로만 잡힌다 — 계획에는 남지만 WinUI
//!   층은 붙이지 않는다(`plan::PlanNode::enter` 문서 참고).
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
