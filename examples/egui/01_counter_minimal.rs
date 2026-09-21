//! egui 예제 01 — 최소 카운터 (eframe + `Ctx` 소유)
//!
//! **언제 쓰나**: 가장 작은 실행 골격. "컴포넌트는 플랫폼을 모른다"는 계약을 눈으로
//! 확인하는 출발점이다.
//!
//! **핵심**
//! - `Ctx`(아레나 = 상태)는 **앱이 소유**한다. 프레임마다 새로 만들면 상태가 사라진다.
//! - 한 프레임은 `frame::<C>` → `render_fast` 두 줄이다.
//! - `CounterProps::default()`가 있으니 `mount!` 때 쓰던 컴포넌트를 그대로 그린다.
//!
//! **주의**: `eframe`은 `egui`와 마이너 버전을 같이 간다(`elm-magic-egui`는 egui 0.36).
//! 버전이 맞지 않으면 `egui::Context`를 직접 소유하고 `Context::run_ui`를 쓰는
//! 방식(예제 18)을 쓴다.

use elm_magic::prelude::*;

elm_magic::view! {
    fn Counter(n = 0) {
        <Col class="root">
            "Count: {n}"
            <Row>
                <Button on_click={n -= 1}>"-"</Button>
                <Button on_click={n += 1}>"+"</Button>
            </Row>
        </Col>
    }
}

fn main() -> eframe::Result<()> {
    // 상태 아레나를 앱 수명 동안 유지한다.
    let mut ctx = Ctx::default();

    eframe::run_simple_native(
        "elm-magic counter",
        eframe::NativeOptions::default(),
        move |egui_ctx, _frame| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                // 1) 상태 → Element 트리
                let tree = elm_magic::frame::<Counter>(&mut ctx, &CounterProps::default());
                // 2) 트리 → egui (앱 경로: Pass 기록을 채우지 않는다)
                elm_magic_egui::render_fast(ui, &tree, &mut ctx.arena);
            });
        },
    )
}
