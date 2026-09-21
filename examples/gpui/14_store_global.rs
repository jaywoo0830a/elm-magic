//! gpui 예제 14 — `#[store]` 전역 상태 (화면 여러 개가 같은 값을 본다)
//!
//! **언제 쓰나**: 헤더/사이드바/상태바가 같은 값을 보거나, 여러 `ElmView`가 같은
//! 세션 정보를 공유할 때. gpui의 엔티티 트리와 **별개로** 존재하는 진실이다.
//!
//! **규칙** (`tests/store.rs`)
//! - `#[store] struct App { .. }`를 컴포넌트보다 위에 선언하고, 본문에서
//!   `app.count`처럼 **소문자 접근자**로 읽고 쓴다.
//! - `#[store]`는 필드 기본값을 못 쓴다(rustc가 구조체로 파싱) → `store_fn!`을 쓴다.
//! - 전역 쓰기는 버전 카운터를 올려 프레임을 다시 돌린다. **어느 `ElmView`에서
//!   썼든 그 엔티티만** 다시 그려지므로, 다른 화면도 갱신하려면 그 엔티티를 깨워야
//!   한다(`cx.notify()` / `Entity::update`).
//!
//! **베스트 패턴**
//! - 전역에는 **진실만**(`dark: bool`, `user_id: u64`). 파생 값은 렌더에서 계산한다.
//! - 화면 간 동기화가 필요하면 "누가 진실을 소유하는가"를 하나로 정한다 —
//!   props(엔티티별 복사)와 전역(공유)을 섞으면 어긋나기 쉽다.
//! - 전역 초기값이 필요하면 `store_fn!`의 기본값으로 표현한다(마운트 훅에서 대입하지 않는다).
//!
//! **주의**: 전역 상태는 **프로세스 전역**이다. 테스트 바이너리 안에서 같은
//! `#[store]` 이름을 쓰면 테스트끼리 영향을 준다(격리 테스트가 따로 있다).

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
        // 두 ElmView가 같은 전역을 본다.
        div()
            .flex()
            .flex_col()
            .child(cx.new(ElmView::<Header>::new))
            .child(cx.new(ElmView::<Body>::new))
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