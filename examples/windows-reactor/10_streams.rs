//! windows-reactor 예제 10 — 스트림 `->` (진행률·로그를 값마다 반영)
//!
//! **업스트림 대응**: `async-state`(진행 상태 갱신). 업스트림은 백그라운드 작업이
//! 메시지마다 상태를 갱신하고, 여기서는 **스트림**이 그 자리를 대신한다 — 펌프만
//! 호스트/어댑터(`drive`)가 맡는다.
//!
//! **언제 쓰나**: 이터레이터/채널이 내보내는 값을 순서대로 상태에 반영할 때.
//!
//! **문법** (사양서 5.4, `tests/stream.rs`)
//! ```text
//! <Button on_click={ upload_progress() -> pct { log = format!("{log}[{pct}]") } }>"go"</Button>
//! ```
//! - `expr -> slot { body }` — 값마다 `slot`에 쓰고 `body`를 실행한다.
//! - `on_message(source, m) { .. }` — 스트림 **구독**(예제 11).
//!
//! **누가 펌프하나**: 어댑터가 아니다. Reactor에서는 `ElmView::update`가
//! `drive::run`을 부르므로 **메시지 하나마다 값 하나씩** 흘러간다. 더 빨리
//! 당기고 싶으면 `drive::pump_streams(&mut view.ctx_mut())`를 직접 부른다.
//! 소진된 태스크는 자동으로 버려진다(반환값 `false`).
//!
//! **주의**
//! - 값마다 **본문 전체가 다시 실행**된다 — 값이 많으면 화면이 무거워진다.
//!   자주 오는 스트림은 작은 `ElmView`로 분리한다(예제 04).
//! - 스트림은 UI 스레드에서 펌프되므로 **블로킹 이터레이터를 쓰면 UI가 멈춘다**.
//! - `format!("{}", log)`처럼 **인자를 토큰으로 넘긴다**. Rust 인라인 캡처
//!   (`format!("{log}")`)는 슬롯 이름을 찾지 못한다 — 매크로는 문자열 리터럴
//!   **안**을 건드리지 않고 슬롯은 `__elm_state_log`로 풀리므로, 본문이 도는
//!   클로저에 `log`라는 지역 변수가 없다(E0425). 화면 텍스트의 `"{log}"`는
//!   요소 자리라서 매크로가 직접 보간하므로 그대로 쓸 수 있다.

use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

fn upload_progress() -> impl Iterator<Item = i32> {
    vec![25, 50, 75, 100].into_iter()
}

elm_magic::view! {
    fn Upload(pct = 0, log = String::new()) {
        <Col>
            <Button on_click={
                upload_progress() -> pct { log = format!("{}[{}]", log, pct) }
            }>"upload"</Button>
            "progress: {pct}%"
            <Progress value={pct as f64 / 100.0} />
            <Text>"{log}"</Text>
        </Col>
    }
}

struct Root;

impl Component for Root {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 스트림");
        View::component::<ElmView<Upload>>(ElmInput::new(UploadProps::default()))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::{drive, plan};

    /// 테스트 하네스의 `pump()`가 하는 일이 곧 앱이 해야 하는 일이다.
    #[test]
    fn stream_feeds_the_progress_bar() {
        let mut app = elm_magic::mount!(Upload);
        app.click("upload");
        app.expect_text("progress: 0%"); // 아직 펌프 전
        app.pump();
        app.expect_text("progress: 25%");
        app.pump();
        app.pump();
        app.pump();
        app.expect_text("progress: 100%");
        app.expect_text("[25][50][75][100]"); // 본문이 값마다 돌았다
    }

    /// 같은 일을 계획 층에서 확인: 진행률이 0..1로 계획에 실린다.
    #[test]
    fn progress_plan_is_a_unit_fraction() {
        let mut ctx = Ctx::new();
        let props = UploadProps {
            pct: Some(50),
            ..Default::default()
        };
        let tree = elm_magic::frame::<Upload>(&mut ctx, &props);
        let (node, pass) = plan(&tree);

        assert_eq!(pass.count("ProgressBar"), 1);
        let bar = node
            .children
            .iter()
            .find(|c| c.control() == "ProgressBar")
            .expect("ProgressBar");
        assert_eq!(bar.fraction, Some(0.5));

        // 스트림 구동 API도 그대로 쓸 수 있다(직접 펌프).
        assert!(!drive::pump_streams(&mut ctx), "등록된 스트림이 없다");
    }
}
