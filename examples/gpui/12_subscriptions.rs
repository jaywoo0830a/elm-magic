//! gpui 예제 12 — 구독: 버스/메시지/네트워크/라우팅을 gpui에서 밀어 넣기
//!
//! **언제 쓰나**: gpui 쪽(또는 다른 스레드)에서 일어난 일을 elm 화면에 반영할 때.
//! egui와 **문법은 같고**, 값이 밖에서 들어오는 통로만 다르다.
//!
//! **통로 (`elm_magic::Arena`의 공개 API)**
//! - `bus.emit(Name)` — 컴포넌트 안에서. 밖에서는 `arena.emit("Refresh")`.
//! - `arena.set_online(bool)` — 네트워크 상태 변화 → `on_net_change`.
//! - `arena.navigate(value)` / `navigate_path("/users/42")` → `on_navigate`.
//! - `on_message(source, m) { .. }` — 스트림 구독. 스트림은 `arena.take_streams()`로
//!   앱이 펌프해야 한다(예제 11의 `drive_effects`와 같은 원리).
//!
//! **gpui에서 밖에서 밀어 넣는 방법**
//! ```ignore
//! view.update(cx, |view, cx| {
//!     view.ctx_mut().arena.emit("Refresh");
//!     cx.notify();                 // 그 엘엠뷰만 다시 그린다
//! });
//! ```
//! `ElmView::ctx_mut()`이 상태 아레나를 그대로 준다 — 어댑터가 이 통로를 열어 둔 이유다.
//!
//! **베스트 패턴**
//! - gpui 이벤트(액션/타이머/다른 엔티티)는 **전부 이 통로로** 들어오게 하고,
//!   elm 쪽은 `on_event`/`on_message`만 알게 한다 → 화면이 플랫폼을 모른다.
//! - 이벤트 이름은 동사구로. 문자열이므로 오타는 조용한 no-op이 된다.
//! - 네트워크 상태는 `on_net_change` 안에서 `net.is_online()`으로 읽는다(스냅샷 복사).
//!
//! **주의**: 핸들러는 **다음 프레임**에 실행된다(emit → 큐 → 프레임). 그래서
//! `cx.notify()`를 함께 불러야 즉시 반영된다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

fn room_messages(room: String) -> Vec<String> {
    vec![format!("live-{room}")]
}

elm_magic::view! {
    fn StatusBar(online = true, hits = 0, msgs: Vec<String> = vec![]) {
        on_event(Refresh) { hits += 1 }
        on_net_change { online = net.is_online(); }
        on_message(room_messages(String::from("lobby")), m) { msgs.push(m); }
        <Col>
            "online: {online}"
            "hits: {hits}"
            <For each={msgs} as={m}>
                <Row>"{m}"</Row>
            </For>
        </Col>
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Route { Home, User(i32) }

elm_magic::view! {
    fn Router(route: Route = Route::Home) {
        on_navigate(|r| route = r)
        <Col>"{route:?}"</Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = cx.new(ElmView::<StatusBar>::new);
        let router = cx.new(ElmView::<Router>::new);

        // 예: 다른 gpui 코드에서 이벤트를 밀어 넣는다.
        //   status.update(cx, |view, cx| {
        //       view.ctx_mut().arena.emit("Refresh");
        //       view.ctx_mut().arena.set_online(false);
        //       cx.notify();
        //   });

        div().flex().flex_col().gap_3().child(status).child(router)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_and_net_change_update_state() {
        let mut app = elm_magic::mount!(StatusBar);
        app.assert_text("hits: 0");
        app.emit("Refresh");
        app.assert_text("hits: 1");

        app.set_online(false);
        app.assert_text("online: false");
    }

    #[test]
    fn navigation_carries_a_typed_route() {
        let mut app = elm_magic::mount!(Router);
        app.navigate_value(Route::User(42));
        app.assert_text("User(42)");
    }
}