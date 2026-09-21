//! egui 예제 10 — 스트림 `->` (진행률처럼 값이 여러 번 오는 것)
//!
//! **언제 쓰나**: 업로드 진행률, 로그 스트림, 웹소켓 메시지처럼 **한 번에 끝나지
//! 않는** 값. `<-`(효과)는 한 값으로 끝나는 future이고, `->`(스트림)는 값마다 슬롯을
//! 갱신하고 본문을 실행한다.
//!
//! **문법**
//! - `expr -> slot { body }` — `expr`은 `IntoIterator`(또는 스트림 소스),
//!   값이 올 때마다 `slot`에 들어가고 `body`가 돈다.
//! - 본문에서는 슬롯을 읽을 수 있다(`log = format!("{}[{}]", log, pct)`).
//! - 펌핑은 런타임이 한다 — 헤드리스에서는 `app.pump()`, 실제 앱에서는 프레임마다
//!   `Ctx`가 처리한다.
//!
//! **베스트 패턴**
//! - 스트림은 **인스턴스 수명에 묶인다**: 컴포넌트가 unmount되면 스트림도 스스로
//!   끝나 죽은 슬롯을 오염시키지 않는다(`tests/state.rs`가 계약).
//! - 값이 여러 번 오면 매번 재렌더된다 — 무거운 계산은 본문이 아니라 파생 값으로
//!   빼 둔다.
//!
//! **주의**: `->`의 소스 함수 이름은 `mock_stream!`의 키다. 테스트에서 값을
//! 결정적으로 만들려면 `elm_magic::mock_stream!(app, upload_progress, [25, 50, 100])`.

use elm_magic::prelude::*;

fn upload_progress() -> impl Iterator<Item = i32> {
    vec![25, 50, 75, 100].into_iter()
}

elm_magic::view! {
    fn Upload(pct = 0, log = String::new()) {
        <Col class="upload">
            <Button on_click={
                upload_progress() -> pct { log = format!("{}[{}]", log, pct) }
            }>"upload"</Button>
            <Progress value={pct as f64 / 100.0} />
            "progress: {pct}%"
            <Text class="muted">"{log}"</Text>
        </Col>
    }
}

/// ── 실제 앱에서 스트림을 구동한다 (중요) ────────────────────
///
/// 테스트의 `app.pump()`에 해당한다. **어댑터는 스트림을 펌프하지 않는다** —
/// 앱이 프레임마다 한 번 불러야 값이 흘러간다. 반환값 `false`면 소진된 태스크다.
fn pump_streams(ctx: &mut Ctx) {
    let tasks = ctx.arena.take_streams();
    let mut keep = Vec::new();
    for mut task in tasks {
        if task(&mut ctx.arena) {
            keep.push(task);
        }
    }
    ctx.arena.push_streams(keep);
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

