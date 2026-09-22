//! windows-reactor 예제 03 — 한 창에 여러 `ElmView` / 여러 창
//!
//! **업스트림 대응**: `gallery` · `navigation`(한 창에 여러 컴포넌트) /
//! `window` · `secondary-window`(여러 창). 창을 늘리는 API도 같다 — `App::run_windows([..])`.
//!
//! **언제 쓰나**: 대시보드처럼 독립 화면을 나란히 두거나, 창을 여러 개 띄울 때.
//!
//! **두 가지 방법**
//! 1. **한 창에 여러 컴포넌트**: `View::fragment((a, b))` 또는 컨테이너의 자식으로
//!    `View::component::<ElmView<C>>(..)`를 여러 개 붙인다. 각자 **자기 상태 아레나**를
//!    갖는다(엔티티 하나 = 인스턴스 하나).
//! 2. **여러 창**: `App::run_windows([root_a, root_b])` — 각 항목이 **독립 창**이다
//!    (`View::fragment`로 묶는 것과 다르다: 그건 한 창 안의 묶음).
//!
//! **핵심**
//! - 한쪽의 상태 변화가 다른 쪽을 다시 그리지 않는다 — 과잉 렌더가 없다.
//! - 공유가 필요하면 elm `#[store]`(예제 13)를 쓴다. 전역 쓰기는 **그 화면만** 깨우므로,
//!   두 화면이 같은 값을 보려면 각자 다시 그려져야 한다.
//! - 창마다 `ElmView`가 따로 살고, 각 창이 자기 메시지 큐를 갖는다(Reactor 보장).

use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    fn Counter(n = 0) {
        <Col>
            "Count: {n}"
            <Button on_click={n += 1}>"+"</Button>
        </Col>
    }
}

elm_magic::view! {
    fn Clock(ticks = 0) {
        on_tick(1s) { ticks += 1 }
        <Col>"ticks: {ticks}"</Col>
    }
}

/// 한 창에 세 화면 (카운터 둘 + 시계).
struct OneWindow;

impl Component for OneWindow {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 여러 뷰");
        // `View::fragment`는 레이아웃 컨테이너를 만들지 않는다 (elm `<>`와 같은 뜻).
        View::fragment((
            View::component::<ElmView<Counter>>(ElmInput::new(CounterProps::default())),
            View::component::<ElmView<Counter>>(ElmInput::new(CounterProps::default())),
            View::component::<ElmView<Clock>>(ElmInput::new(ClockProps::default())),
        ))
    }
}

fn main() {
    // ① 한 창: 여러 ElmView가 각자 상태를 센다.
    App::run_component::<OneWindow>(()).expect("Reactor 실행 실패");

    // ② 두 창: 각 항목이 독립 창이다.
    // App::run_windows([
    //     View::component::<ElmView<Counter>>(ElmInput::new(CounterProps::default())),
    //     View::component::<ElmView<Clock>>(ElmInput::new(ClockProps::default())),
    // ])
    // .expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn each_view_owns_its_own_state() {
        // 두 인스턴스는 서로 다른 아레나를 갖는다 — 하나를 바꿔도 다른 하나는 그대로.
        let mut first = Ctx::new();
        let mut second = Ctx::new();
        let props = CounterProps::default();

        let tree = elm_magic::frame::<Counter>(&mut first, &props);
        let (_, pass) = plan(&tree);
        assert!(pass.has_text("Count: 0"));

        // 첫 번째 아레나만 상태를 바꾼다.
        first.arena.mutate::<i32, _>(0, |n| *n = 5);
        let tree = elm_magic::frame::<Counter>(&mut first, &props);
        assert!(plan(&tree).1.has_text("Count: 5"));

        let tree = elm_magic::frame::<Counter>(&mut second, &props);
        assert!(plan(&tree).1.has_text("Count: 0"), "두 번째는 그대로");
    }
}
