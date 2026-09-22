//! windows-reactor 예제 11 — 호스트 상태 ↔ elm (props와 콜백 prop)
//!
//! **업스트림 대응**: `component-input`(props) · `async-state`/`form`(백그라운드 작업) ·
//! `message-box`(호스트가 소유한 결과 표시). 업스트림 관용구 그대로다 — 무거운 일은
//! **호스트**, 결과는 props, 되돌아오는 신호는 **콜백 prop**.
//!
//! **언제 쓰나**: WinUI 쪽에서만 할 수 있는 일(백그라운드 작업, 창/파일/장치 접근)의
//! 결과를 elm 화면에 반영할 때.
//!
//! **두 방향**
//! - **호스트 → elm**: 호스트가 상태를 소유하고 **props**로 내려보낸다. 부모가 새
//!   `ElmInput`을 만들면 `input_changed`가 불려 elm이 다시 그린다(예제 02).
//! - **elm → 호스트**: **콜백 prop**(`fn(i32)`)을 내려보내고, elm은 `on_reload(1)`처럼
//!   부른다. 호스트는 그 안에서 `LocalSender::send`로 **자기 메시지 큐**에 넣는다
//!   (Reactor는 콜백 안에서 `update`를 인라인 실행하지 않는다 — 이벤트 FIFO).
//!
//! **백그라운드 작업**
//! - `ComponentContext::spawn_background(work)`는 `C::Message: Send`를 요구한다.
//!   **호스트의 메시지는 `Send`여야 한다**(여기서는 `HostMessage`).
//! - 이 어댑터의 `ElmMessage`는 `Rc<dyn Fn(..)>`를 나르므로 **`Send`가 아니다** —
//!   그래서 elm 쪽에서 `spawn_background`를 직접 쓸 수 없다. 무거운 일은 **호스트가**
//!   맡고 결과만 props로 내려보내는 것이 정석이다.
//! - `ElementRef<T>`는 **컨트롤용**이라 컴포넌트(`ElmView`) 핸들로는 쓸 수 없다 —
//!   "elm 인스턴스를 밖에서 조작"하는 길은 없다. props/콜백이 유일한 계약이다.
//!
//! **elm 내부에서 외부로 알리기**: `arena.emit("Name")`, `arena.set_online(..)`,
//! `arena.navigate(value)`는 elm 코드에서 쓸 수 있고 `on_event`/`on_net_change`/
//! `on_navigate`가 받는다(예제 12). 호스트가 쓰려면 props로 값을 내려보내는 편이 낫다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

elm_magic::view! {
    fn Panel(status: String, loading: bool, on_reload: fn(i32)) {
        <Col>
            <If when={loading}><Spinner /></If>
            "status: {status}"
            <Button on_click={on_reload(1)}>"다시 불러오기"</Button>
        </Col>
    }
}

#[derive(Clone)]
enum HostMessage {
    /// elm의 콜백 prop이 큐에 넣는다.
    Reload,
    /// 백그라운드 작업의 결과 (Send 가능한 값만).
    Loaded(String),
}

struct Host {
    status: String,
    loading: bool,
}

impl Component for Host {
    type Input = ();
    type Message = HostMessage;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            status: "idle".to_string(),
            loading: false,
        }
    }

    fn update(&mut self, message: HostMessage, context: &ComponentContext<Self>) {
        match message {
            HostMessage::Reload => {
                self.loading = true;
                // UI 스레드 밖에서 도는 작업. 반환 메시지는 Send여야 한다.
                let _ = context.spawn_background(|_cancel| {
                    let value = "loaded".to_string();
                    HostMessage::Loaded(value)
                });
            }
            HostMessage::Loaded(value) => {
                self.status = value;
                self.loading = false;
            }
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 호스트 상태");

        // elm → 호스트 신호: 콜백 prop 안에서 메시지 큐에 넣는다.
        let sender = context.sender();
        let on_reload = Callback::new(move |_arena, _: i32| {
            let _ = sender.send(HostMessage::Reload);
        });

        View::component::<ElmView<Panel>>(ElmInput::new(PanelProps {
            status: Some(self.status.clone()),
            loading: Some(self.loading),
            on_reload: Some(on_reload),
            ..Default::default()
        }))
    }
}

fn main() {
    App::run_component::<Host>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::plan;

    #[test]
    fn props_drive_the_elm_view() {
        let mut ctx = Ctx::new();

        let loading = PanelProps {
            status: Some("idle".to_string()),
            loading: Some(true),
            ..Default::default()
        };
        let tree = elm_magic::frame::<Panel>(&mut ctx, &loading);
        let pass = plan(&tree).1;
        assert_eq!(pass.count("ProgressRing"), 1, "로딩이면 Spinner");
        assert!(pass.has_text("status: idle"));

        let done = PanelProps {
            status: Some("loaded".to_string()),
            loading: Some(false),
            ..Default::default()
        };
        let tree = elm_magic::frame::<Panel>(&mut ctx, &done);
        let pass = plan(&tree).1;
        assert_eq!(pass.count("ProgressRing"), 0);
        assert!(pass.has_text("status: loaded"));
    }

    #[test]
    fn callback_prop_reaches_the_host_closure() {
        use std::cell::Cell;
        use std::rc::Rc;

        let hits = Rc::new(Cell::new(0));
        let seen = Rc::clone(&hits);
        let props = PanelProps {
            status: Some("idle".to_string()),
            loading: Some(false),
            on_reload: Some(Callback::new(move |_arena, value: i32| {
                seen.set(seen.get() + value);
            })),
            ..Default::default()
        };

        let mut app = elm_magic::mount_with::<Panel>(props);
        app.click("다시 불러오기");
        assert_eq!(hits.get(), 1, "elm이 호스트 콜백을 불러야 한다");
    }
}
