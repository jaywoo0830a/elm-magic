//! 구동 헬퍼 — **앱이 돌려야 하는 것들**을 한 곳에 모은다 (플랫폼 독립).
//!
//! elm-magic의 런타임은 렌더만 한다. `<-`(효과), `->`(스트림), `on_key` 핸들러,
//! `on_tick`의 시계는 **호스트가 돌려야** 한다 — 헤드리스 테스트에서는
//! `TestApp::flush`/`advance`/`pump`/`press_key`가 그 역할을 한다.
//!
//! 어댑터 사용자(그리고 다른 어댑터 저자)가 같은 실수를 반복하지 않도록,
//! 그 동작을 공개 함수로 옮겼다. **플랫폼에 의존하지 않으므로 어디서나 쓸 수 있고
//! 테스트된다.**
//!
//! ```ignore
//! // Reactor: 메시지 하나를 처리한 뒤 같은 발행 안에서 결과를 반영한다.
//! fn update(&mut self, message: Msg, _ctx: &ComponentContext<Self>) {
//!     self.handler(&mut self.ctx.arena, message);
//!     elm_magic_windows_reactor::drive::run(&mut self.ctx);
//! }
//! ```

use elm_magic::Ctx;

/// due가 된 효과를 모두 실행한다 (재귀적으로 — 효과가 효과를 낳을 수 있다).
///
/// 반환값은 "무언가 실행됐는가" — 참이면 다시 그릴 필요가 있다.
pub fn due_effects(ctx: &mut Ctx) -> bool {
    let mut ran = false;
    // 효과가 효과를 낳는 경우를 위해, 큐가 빌 때까지 돈다(테스트와 같은 상한).
    for _ in 0..1000 {
        let due = ctx.arena.take_due();
        if due.is_empty() {
            break;
        }
        for effect in due {
            effect(&mut ctx.arena);
        }
        ran = true;
    }
    ran
}

/// 살아 있는 스트림에서 값 하나씩 당긴다 (소진된 태스크는 버린다).
pub fn pump_streams(ctx: &mut Ctx) -> bool {
    let tasks = ctx.arena.take_streams();
    let mut keep = Vec::new();
    let mut ran = false;
    for mut task in tasks {
        if task(&mut ctx.arena) {
            keep.push(task);
            ran = true;
        }
    }
    ctx.arena.push_streams(keep);
    ran
}

/// 시계를 `now_ms`로 맞추고 효과와 스트림을 구동한다.
///
/// `<- f() after 300ms`는 시계가 밀려야 due가 된다 — 시계를 앱이 관리한다는
/// 사실이 여기 드러난다(`on_tick`의 주기도 같은 시계를 쓴다).
pub fn run_at(ctx: &mut Ctx, now_ms: u64) -> bool {
    ctx.arena.set_now(now_ms);
    run(ctx)
}

/// 효과와 스트림을 모두 구동한다 (시계는 그대로).
pub fn run(ctx: &mut Ctx) -> bool {
    let effects = due_effects(ctx);
    let streams = pump_streams(ctx);
    effects || streams
}

/// `on_key("Ctrl+S")`가 등록한 핸들러를 찾아 실행한다.
///
/// 핸들러는 마지막 프레임이 `Ctx::keys`에 남긴 것이다. 호스트(Reactor의 키
/// 바인딩, egui/gpui의 키 이벤트)가 그 목록을 자기 키 이벤트에 연결해야 한다 —
/// 반환값은 "핸들러를 찾았는가"다.
pub fn dispatch_key(ctx: &mut Ctx, key: &str) -> bool {
    let handler = ctx
        .keys
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, h)| h.clone());
    match handler {
        Some(h) => {
            h(&mut ctx.arena);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    // 전역 glob(`use super::*`)은 `elm_magic::run`과 이름이 겹쳐 모호해진다 —
    // 필요한 것만 명시적으로 가져온다.
    use super::{dispatch_key, run, run_at};
    use elm_magic::Ctx;

    async fn loaded() -> String {
        "done".to_string()
    }

    elm_magic::view! {
        fn Effects(status = String::from("idle")) {
            on_mount { status <- loaded() }
            <Text>"status: {status}"</Text>
        }
    }

    elm_magic::view! {
        fn Delayed(status = String::from("idle")) {
            on_mount { status <- loaded() after 300ms }
            <Text>"status: {status}"</Text>
        }
    }

    elm_magic::view! {
        fn Keys(hit = 0) {
            on_key("Ctrl+S") { hit += 1 }
            <Text>"hit: {hit}"</Text>
        }
    }

    fn render<C: elm_magic::Component>(ctx: &mut Ctx, props: &C::Props) -> Vec<String> {
        elm_magic::frame::<C>(ctx, props).texts()
    }

    #[test]
    fn run_executes_due_effects() {
        let mut ctx = Ctx::new();
        let props = EffectsProps::default();
        let first = render::<Effects>(&mut ctx, &props);
        assert!(
            first.contains(&"status: idle".to_string()),
            "첫 프레임: {first:?}"
        );

        assert!(run(&mut ctx), "due 효과가 실행돼야 한다");
        let after = render::<Effects>(&mut ctx, &props);
        assert!(
            after.contains(&"status: done".to_string()),
            "구동 후: {after:?}"
        );
    }

    #[test]
    fn run_at_moves_the_clock_for_delayed_effects() {
        let mut ctx = Ctx::new();
        let props = DelayedProps::default();
        let first = render::<Delayed>(&mut ctx, &props);

        assert!(!run(&mut ctx), "아직 due가 아니다 (after 300ms): {first:?}");
        assert!(run_at(&mut ctx, 300), "시계를 밀면 실행된다");
        let after = render::<Delayed>(&mut ctx, &props);
        assert!(
            after.contains(&"status: done".to_string()),
            "구동 후: {after:?}"
        );
    }

    #[test]
    fn dispatch_key_runs_the_registered_handler() {
        let mut ctx = Ctx::new();
        let props = KeysProps::default();
        let _ = render::<Keys>(&mut ctx, &props);

        assert!(dispatch_key(&mut ctx, "Ctrl+S"), "on_key가 등록돼야 한다");
        assert!(!dispatch_key(&mut ctx, "Ctrl+Q"), "없는 키는 false");
        assert!(render::<Keys>(&mut ctx, &props).contains(&"hit: 1".to_string()));
    }

    #[test]
    fn run_is_idempotent_when_nothing_is_pending() {
        let mut ctx = Ctx::new();
        assert!(!run(&mut ctx), "큐가 비면 false");
    }
}
