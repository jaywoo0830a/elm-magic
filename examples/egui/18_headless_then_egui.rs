//! egui 예제 18 — 헤드리스 계약 → egui 실측 (2단 검증)
//!
//! **언제 쓰나**: 상태 로직은 빠르게, 실제 클릭/렌더는 한 번만 확인하고 싶을 때.
//!
//! **2단 전략**
//! 1. **헤드리스**(`mount!`) — 상태·이벤트·텍스트를 결정적으로 검증한다. 렌더러도
//!    런타임도 필요 없다(0.8초짜리 테스트 290개가 이 계약 위에 서 있다).
//! 2. **어댑터**(`egui::Context::run_ui`) — 같은 트리를 실제 egui에 그려
//!    (a) 페인트된 텍스트와 (b) `Pass`의 버튼 rect로 **진짜 입력**을 보낸다.
//!
//! **왜 이 순서인가**: 1단에서 깨지면 2단은 볼 필요가 없다. 2단은 "헤드리스에만
//! 있는 계약"을 막고, 의도된 발산(예: `disabled` 버튼은 egui가 클릭을 무시)을
//! 드러낸다 — `crates/elm-magic-egui/tests/adapter.rs`가 그 예들이다.
//!
//! **주의**: 헤드리스에는 페인터가 없으므로 `FullOutput.textures_delta.clear()`로
//! 폰트 아틀라스 델타를 소비해야 한다(안 하면 다음 프레임이 무거워진다).

use elm_magic::Component;
use elm_magic::prelude::*;

elm_magic::view! {
    fn Counter(n = 0) {
        <Col>
            "Count: {n}"
            <Button on_click={n += 1}>"+"</Button>
            <Button on_click={n -= 1} disabled={n == 0}>"-"</Button>
        </Col>
    }
}

fn raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    }
}

/// 한 프레임 그리기: (페인트 출력, 버튼 rect가 담긴 Pass)
fn frame<C: Component>(
    ctx: &egui::Context,
    app: &mut elm_magic::testing::TestApp<C>,
    input: egui::RawInput,
) -> (egui::FullOutput, elm_magic_egui::Pass) {
    let tree = app.element().clone();
    let mut pass = None;
    let mut out = ctx.run_ui(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            pass = Some(elm_magic_egui::render(ui, &tree, &mut app.ctx.arena));
        });
    });
    out.textures_delta.clear(); // 헤드리스: 페인터가 없다
    (out, pass.expect("adapter did not run"))
}

fn click_at(pos: egui::Pos2) -> egui::RawInput {
    let mut i = raw_input();
    i.events = vec![
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        },
    ];
    i
}

fn painted_text(out: &egui::FullOutput) -> String {
    out.shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1단: 상태 계약 (가장 빠르고 먼저 깨진다).
    #[test]
    fn headless_contract() {
        let mut app = elm_magic::mount!(Counter);
        app.expect_text("Count: 0");
        app.click("+");
        app.expect_text("Count: 1");
    }

    /// 2단: 실제 egui 입력으로 같은 변화를 확인한다.
    #[test]
    fn egui_input_reaches_the_arena() {
        let mut app = elm_magic::mount::<Counter>();
        let ctx = egui::Context::default();

        let (_, pass) = frame(&ctx, &mut app, raw_input());
        let (_, plus) = pass
            .buttons
            .iter()
            .find(|(label, _)| label == "+")
            .expect("'+' 버튼이 그려졌다");

        let _ = frame(&ctx, &mut app, click_at(plus.rect.center()));
        app.refresh();

        let (out, _) = frame(&ctx, &mut app, raw_input());
        assert!(painted_text(&out).contains("Count: 1"));
    }
}
