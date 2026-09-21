//! egui 예제 11 — 구독: `on_message` / `on_event` / `on_net_change` / `on_navigate`
//!
//! **언제 쓰나**: 컴포넌트 바깥의 세계(다른 컴포넌트, 네트워크 상태, 라우터)가
//! 바뀔 때 상태를 맞춰야 할 때. 타이머/입력이 아니라 **실제 변화에만** 반응한다.
//!
//! **도구**
//! - `on_message(source, m) { .. }` — 스트림 구독. 메시지마다 상태를 갱신한다.
//! - `on_event(Name) { .. }` + `bus.emit(Name)` — 크로스 컴포넌트 이벤트 버스.
//!   `Name`은 **식별자 토큰**이고 문자열로 이름 붙는다(`Refresh` == `"Refresh"`).
//! - `on_net_change { online = net.is_online(); }` — 온라인/오프라인 전환.
//! - `on_navigate(|p| path = p)` / `on_navigate(|r| route = r)` — 경로(문자열) 또는
//!   타입 있는 라우트 값. 후자는 `nav_take`로 **소유 복제본**을 받는다.
//!
//! **베스트 패턴**
//! - 버스 이벤트는 **동사구**로 이름 짓고(문자열이 되므로 오타가 조용히 no-op이 될 수
//!   있다) 테스트에서 `app.emit("Refresh")`로 직접 쏜다.
//! - `on_message`는 "서버가 진실"인 상태를 담기 좋다(예: 채팅 로그).
//! - 읽기 전용 파생(온라인 여부)은 구독 결과로 슬롯에 복사해 두면 렌더가 순수해진다.
//!
//! **주의**: 이 핸들러들은 **다음 프레임**에 실행된다(emit → 큐 → 프레임).
//! 그래서 `flush()`/`pump()` 뒤에 단언한다.

use elm_magic::prelude::*;

fn room_messages(room: String) -> Vec<String> {
    vec![format!("live-{room}")]
}

elm_magic::view! {
    fn Toolbar() {
        <Row>
            <Button on_click={bus.emit(Refresh)}>"refresh"</Button>
            <Button on_click={bus.emit(SyncNow)}>"sync"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn Chat(room = String::from("lobby"), msgs: Vec<String> = vec![]) {
        on_message(room_messages(room.clone()), m) { msgs.push(m); }
        <Col>
            "room: {room}"
            <For each={msgs} as={m}>
                <Row>"{m}"</Row>
            </For>
        </Col>
    }
}

elm_magic::view! {
    fn StatusBar(online = true, hits = 0) {
        on_net_change { online = net.is_online(); }
        on_event(Refresh) { hits += 1 }
        on_event(SyncNow) { hits += 1 }
        <Col>
            "online: {online}"
            "hits: {hits}"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_subscription_pumps_into_state() {
        let mut app = elm_magic::mount_with::<Chat>(ChatProps {
            room: Some("dev".into()),
            ..Default::default()
        });
        app.pump();
        app.assert_text("live-dev");
    }

    #[test]
    fn bus_reaches_another_component() {
        let mut app = elm_magic::mount!(StatusBar);
        app.assert_text("hits: 0");
        app.emit("Refresh");
        app.assert_text("hits: 1");
        app.emit("SyncNow");
        app.assert_text("hits: 2");
    }

    #[test]
    fn net_and_route_changes() {
        let mut app = elm_magic::mount!(StatusBar);
        app.set_online(false);
        app.assert_text("online: false");

        let mut app = elm_magic::mount!(Router);
        app.navigate_value(Route::User(42));
        app.assert_text("User(42)");
    }
}
