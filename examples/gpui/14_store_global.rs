//! gpui 예제 14 — `#[store]` 상태 (한 화면 트리 안에서 공유)
//!
//! **언제 쓰나**: 헤더/사이드바/상태바가 같은 값을 볼 때. props로 내려보내는
//! "prop drilling"을 없앤다.
//!
//! **규칙** (`tests/store.rs`)
//! - `#[store] struct App { .. }`를 컴포넌트보다 위에 선언하고, 본문에서
//!   `app.count`처럼 **소문자 접근자**로 읽고 쓴다.
//! - `#[store]`는 필드 기본값을 못 쓴다(rustc가 구조체로 파싱) → `store_fn!`을 쓴다.
//! - 전역 쓰기는 버전 카운터를 올려 **그 엔티티**를 다시 그린다.
//!
//! **중요 — "전역"의 범위**: store는 **아레나마다 하나**다(`Arena.stores`).
//! 한 `ElmView`(= 한 `Ctx`) 안의 모든 컴포넌트가 공유하지만, **별개의
//! `ElmView` 인스턴스끼리는 공유하지 않는다**. 그래서:
//! - 공유가 필요하면 **하나의 루트 컴포넌트**(아래 `Root`)로 묶어 `ElmView<Root>` 하나만
//!   붙인다 — 이 예제의 방식이다.
//! - 화면을 여러 `ElmView`로 나눠야 한다면 공유 값은 **props로 내려보낸다**(예제 02).
//!
//! **베스트 패턴**
//! - store에는 **진실만**(`dark: bool`, `user_id: u64`). 파생 값은 렌더에서 계산한다.
//! - 초기값이 필요하면 `store_fn!`의 기본값으로 표현한다(마운트 훅에서 대입하지 않는다).
//! - 테스트는 격리된다 — 테스트마다 새 아레나가 만들어지므로 store 이름이 겹쳐도 안전하다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

#[store]
struct App {
    dark: bool,
}

elm_magic::store_fn! {
    Session { user_id: u64 = 0, token: String = String::new() }
}

elm_magic::view! {
    fn Header() {
        <Row>
            "dark: {app.dark}"
            "user: {session.user_id}"
            <Button on_click={app.dark = !app.dark}>"테마"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn Body() {
        <Col>
            "token: {session.token}"
            <Button on_click={session.token = String::from("t-1")}>"로그인"</Button>
        </Col>
    }
}

elm_magic::view! {
    fn Root() {
        <Col>
            <Header />
            <Body />
        </Col>
    }
}

struct App2;
impl Render for App2 {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 공유가 필요하면 **루트 컴포넌트 하나**를 붙인다.
        //   `ElmView<Header>`와 `ElmView<Body>`를 따로 붙이면 아레나가 둘이라
        //   store도 둘이다 — 서로의 값을 보지 못한다.
        div().flex().flex_col().child(cx.new(ElmView::<Root>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_is_shared_and_has_defaults() {
        let mut app = elm_magic::mount!(Root);
        app.assert_text("dark: false");
        app.assert_text("user: 0"); // store_fn! 기본값
        app.click("로그인");
        app.assert_text("token: t-1");
        app.click("테마");
        app.assert_text("dark: true");
    }
}