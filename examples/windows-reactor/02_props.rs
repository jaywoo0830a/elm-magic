//! windows-reactor 예제 02 — props (`ElmInput`의 동일성 비교)
//!
//! **언제 쓰나**: 화면을 부모 상태에서 분리해 재사용할 때.
//!
//! **왜 래퍼가 필요한가** (`crates/elm-magic-windows-reactor/src/winui.rs`)
//! - Reactor의 `Component::Input`은 `Clone + PartialEq + 'static`을 요구한다.
//! - elm-magic의 생성된 `…Props`는 `Clone`만 파생하고 `Default`를 직접 구현한다
//!   (`PartialEq` 없음). 그래서 [`ElmInput`]이 `Rc<P>`를 들고 **동일성 비교**를 한다.
//! - 결과: 부모가 **새 props**를 만들어 넘기면 `input_changed`가 불려 elm이 다시
//!   그린다. 같은 `ElmInput`을 복제해 넘기면(`ElmInput::clone`) 아무 일도 없다.
//!
//! **베스트 패턴**
//! - 자식 화면은 `View::component::<ElmView<Child>>(ElmInput::new(props))`로 붙인다.
//!   부모가 매 프레임 새 `ElmInput`을 만들면 자식은 매번 갱신된다 — 자주 바뀌지
//!   않는 props라면 부모가 **캐시**(필드에 담아 두기)하는 편이 싸다.
//! - 필수 prop(기본값 없는 매개변수)을 `None`으로 두면 첫 렌더에서 panic한다 —
//!   `Some(..)`을 채워야 한다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{plan, ElmInput, ElmView};
use windows_reactor::{Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    // `title`은 필수 prop, `count`는 기본값 0.
    fn Panel(title: String, count = 0) {
        <Col>
            <Strong>"{title}"</Strong>
            "count: {count}"
            <Button on_click={count += 1}>"inc"</Button>
        </Col>
    }
}

/// 부모 컴포넌트 — props를 소유하고 자식 `ElmView`를 붙인다.
struct Dashboard {
    title: String,
}

impl Component for Dashboard {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            title: "패널".to_string(),
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        // 창 제목도 여기서 선언한다 — elm 컴포넌트는 플랫폼을 모르므로 어댑터가
        // 대신 정하지 않는다(예제 20 참고).
        context.window_title("elm-magic — props");

        let props = PanelProps {
            title: Some(self.title.clone()),
            count: None, // 기본값 0
            ..Default::default()
        };
        View::component::<ElmView<Panel>>(ElmInput::new(props))
    }
}

fn main() {
    windows_reactor::App::run_component::<Dashboard>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::Pass;

    fn panel_pass(props: PanelProps) -> Pass {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Panel>(&mut ctx, &props);
        plan(&tree).1
    }

    #[test]
    fn required_props_are_read_from_the_input() {
        let pass = panel_pass(PanelProps {
            title: Some("제목".to_string()),
            count: Some(7),
            ..Default::default()
        });
        assert!(pass.has_text("제목"));
        assert!(pass.has_text("count: 7"));
    }

    #[test]
    fn missing_optional_props_use_declared_defaults() {
        let pass = panel_pass(PanelProps {
            title: Some("제목".to_string()),
            ..Default::default()
        });
        assert!(pass.has_text("count: 0"), "기본값이 쓰여야 한다");
    }

    #[test]
    fn input_identity_drives_input_changed() {
        let a = ElmInput::new(1u32);
        let b = a.clone();
        assert_eq!(a, b, "같은 Rc면 같다 — input_changed가 불리지 않는다");
        assert_ne!(a, ElmInput::new(1u32), "새로 만들면 다르다");
    }
}
