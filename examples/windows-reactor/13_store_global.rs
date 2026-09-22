//! windows-reactor 예제 13 — `#[store]` 상태 (한 ElmView 트리 안에서 공유)
//!
//! **언제 쓰나**: 헤더/사이드바/상태바가 같은 값을 볼 때. props로 내려보내는
//! "prop drilling"을 없앤다.
//!
//! **규칙** (`tests/store.rs`)
//! - `#[store] struct App { .. }`를 컴포넌트보다 위에 선언하고, 본문에서
//!   `app.count`처럼 **소문자 접근자**로 읽고 쓴다.
//! - `#[store]`는 필드 기본값을 못 쓴다(rustc가 구조체로 파싱) → `store_fn!`을 쓴다.
//! - 전역 쓰기는 버전 카운터를 올려 **그 컴포넌트**를 다시 그린다.
//!
//! **중요 — "전역"의 범위**: store는 **아레나마다 하나**다(`Arena.stores`).
//! 한 `ElmView`(= 한 `Ctx`) 안의 모든 컴포넌트가 공유하지만, **별개의
//! `ElmView` 인스턴스끼리는 공유하지 않는다**. 그래서:
//! - 공유가 필요하면 **하나의 루트 컴포넌트**(아래 `Root`)로 묶어
//!   `ElmView<Root>` 하나만 붙인다 — 이 예제의 방식이다.
//! - 화면을 여러 `ElmView`로 나눠야 한다면 공유 값은 **props로 내려보낸다**(예제 11).
//!
//! **호스트에서 값을 밀어 넣어야 한다면**
//! - 권장: props(예제 11). 키 문자열을 몰라도 되고 타입도 안전하다.
//! - 아레나를 직접 만질 수 있을 때만: `arena.store_set::<T>("모듈::Store.필드", 값)`.
//!   키는 `concat!(module_path!(), "::{store}.{field}")` 형식이다
//!   (`crates/elm-magic-macros/src/store.rs`) — 오타는 조용한 no-op이 된다.
//!
//! **업스트림 대응** (`crates/samples/reactor/context`)
//! - Reactor에는 **자기 `Context`**가 있다: `Context::<String>::new(..)` +
//!   `View::provide(&ctx, value, child)` + `ViewContext::use_context(&ctx)`. 그것은
//!   **Reactor 컴포넌트 사이**의 공유이고, `#[store]`는 **elm 아레나 안**의 공유다 —
//!   층이 다르다. 호스트 쪽 전역(테마·로그인 사용자)은 Reactor `Context`,
//!   화면 안의 전역은 `#[store]`로 두는 것이 정석이다.
//!
//! **테스트 격리**: 테스트마다 새 아레나가 만들어지므로 store 이름이 겹쳐도 안전하다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
// store 이름(`App`)과 Reactor의 `App`이 겹친다 — store가 이 모듈에 `struct App`을
// 만들므로 `use windows_reactor::App`과 충돌한다(E0255). 그래서 경로로 쓴다.
use windows_reactor::{Component, ComponentContext, View, ViewContext};

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

// 공유가 필요하므로 **루트 컴포넌트 하나**만 붙인다.
elm_magic::view! {
    fn Root() {
        <Col>
            <Header />
            <Body />
        </Col>
    }
}

struct Shell;

impl Component for Shell {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — store");
        // `ElmView<Header>`와 `ElmView<Body>`를 따로 붙이면 아레나가 둘이라 store도
        // 둘이다 — 서로의 값을 보지 못한다. 그래서 루트 하나를 붙인다.
        View::component::<ElmView<Root>>(ElmInput::new(RootProps::default()))
    }
}

fn main() {
    windows_reactor::App::run_component::<Shell>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn store_is_shared_inside_one_component_tree() {
        let mut app = elm_magic::mount!(Root);
        app.assert_text("dark: false");
        app.assert_text("user: 0"); // store_fn! 기본값
        app.assert_text("token: ");

        app.click("로그인"); // Body가 전역을 바꾼다
        app.assert_text("token: t-1");

        app.click("테마"); // Header가 전역을 바꾼다
        app.assert_text("dark: true");
    }

    #[test]
    fn separate_instances_do_not_share_the_store() {
        // 별개의 아레나 → store도 별개. (공유가 필요하면 루트로 묶는다.)
        let mut body_ctx = Ctx::new();
        let mut header_ctx = Ctx::new();

        let tree = elm_magic::frame::<Body>(&mut body_ctx, &BodyProps::default());
        assert!(plan(&tree).1.has_text("token: "));

        let tree = elm_magic::frame::<Header>(&mut header_ctx, &HeaderProps::default());
        let pass = plan(&tree).1;
        assert!(pass.has_text("dark: false"));
        assert!(pass.has_text("user: 0"));
    }

    #[test]
    fn store_writes_show_up_in_the_same_tree() {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<Root>(&mut ctx, &RootProps::default());
        let pass = plan(&tree).1;
        // 헤더와 본문이 같은 store를 읽는다.
        assert!(pass.has_text("dark: false"));
        assert!(pass.has_text("token: "));
    }
}
