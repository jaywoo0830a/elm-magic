//! windows-reactor 예제 01 — 최소 카운터 (`ElmView` + `App::run_component`)
//!
//! **언제 쓰나**: WinUI 3 앱에 elm-magic 화면 하나를 붙이는 최소 골격.
//! egui/gpui 예제 01과 **같은 컴포넌트**가 그대로 동작한다 — 컴포넌트는 플랫폼을 모른다.
//!
//! **핵심**
//! - `ElmView<C>`가 `windows_reactor::Component`를 구현한다 → `App::run_component`
//!   한 줄로 창이 뜬다.
//! - props는 [`ElmInput`]으로 감싼다. Reactor의 `Input`은 `Clone + PartialEq`를
//!   요구하는데 elm의 `…Props`는 `Clone`만 파생하므로, 어댑터가 **동일성 비교**
//!   래퍼를 제공한다(예제 02).
//! - 상태 아레나는 `ElmView`가 소유한다. **엔티티 하나 = 컴포넌트 인스턴스 하나**다.
//!
//! **주의**
//! - `css!`는 이 백엔드에서 **효과가 없다** — 어댑터가 스타일 계층을 지원하지 않는다.
//!   간격/색은 WinUI 테마·리소스로, 세부 조정은 `<Raw>`로 한다(예제 16, 08).
//! - `windows-reactor`는 Windows 전용(WASDK)이다. 이 파일은 Windows에서만 컴파일된다.
//!
//! **의존성**
//! ```toml
//! [dependencies]
//! elm-magic = "0.8.5"
//! elm-magic-windows-reactor = "0.8.5"
//! windows-reactor = "0.100"
//! ```

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::App;

elm_magic::view! {
    fn Counter(n = 0) {
        <Col>
            "Count: {n}"
            <Row>
                <Button on_click={n -= 1}>"-"</Button>
                <Button on_click={n += 1}>"+"</Button>
            </Row>
        </Col>
    }
}

fn main() {
    // 창 하나 + 컴포넌트 하나. WinUI 메시지 루프가 닫힐 때까지 블록된다.
    App::run_component::<ElmView<Counter>>(ElmInput::new(CounterProps::default()))
        .expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    /// 계획 층은 플랫폼 독립이라 **리눅스 CI에서도 돈다** (tests/plan.rs와 같은 방식).
    #[test]
    fn counter_maps_to_a_stackpanel_with_two_buttons() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Counter>(&mut ctx, &CounterProps::default());
        let (node, pass) = plan(&tree);

        assert_eq!(node.control(), "StackPanel");
        assert_eq!(pass.count("Button"), 2);
        assert_eq!(pass.labeled, vec!["-", "+"]);
        assert!(pass.has_text("Count: 0"));
    }
}
