//! gpui 예제 10 — `Pass`로 어댑터가 본 것을 검증 (`styles` / `labeled` / `inputs`)
//!
//! **언제 쓰나**: 렌더 결과를 디버깅할 때. gpui는 egui처럼 "페인트 명령"을 쉽게
//! 꺼내 볼 수 없으므로, 어댑터가 남기는 `Pass`가 사실상 유일한 창이다.
//!
//! **`Pass`의 계약** (`crates/elm-magic-gpui/src/lib.rs`)
//! - `styles: Vec<(&'static str, ResolvedStyle)>` — **스타일이 비어 있지 않은** 노드만.
//!   `pass.style_of(tag)`로 첫 번째를 꺼낸다. 스타일 회귀는 여기서 잡는다.
//! - `labeled: Vec<String>` — 그려진 버튼·탭·헤더 라벨(순서대로).
//! - `inputs: usize` — 그려진 입력 필드 수(포커스 핸들이 만들어졌다는 신호).
//! - `ElmView::pass()`는 **마지막 프레임**의 기록이다. `ctx()`/`ctx_mut()`으로
//!   아레나를 직접 볼 수도 있다(테스트에서 이펙트를 구동할 때 필요).
//!
//! **베스트 패턴**
//! - 상태 계약은 헤드리스(`mount!`)로, **스타일 전달**은 `Pass`로 확인한다.
//! - 라벨 목록은 E2E 회귀에 유용하다 — 버튼 텍스트가 바뀌면 여기가 깨진다.
//! - `inputs`는 폼 화면에서 "입력이 빠졌다"를 잡는 싼 지표다.
//!
//! **주의**: `Pass`는 **프레임 단위**다. 새 프레임(`render_frame`) 후에 읽어야 한다.

use elm_magic::prelude::*;
use elm_magic_gpui::{ElmView, Pass};
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::css! {
    .panel { gap: 8; padding: 16; bg: surface; radius: 8; }
    .panel__title { color: text_dim; font-size: 18; weight: bold; }
}

elm_magic::view! {
    fn Panel(name = String::new(), n = 0) {
        <Col class="panel">
            <Text class="panel__title">"hello"</Text>
            <Input class="field" value={name.clone()} on_change={name = _} />
            <Button on_click={n += 1}>"ok {n}"</Button>
        </Col>
    }
}

/// 어댑터가 본 것을 그대로 출력한다 — "왜 스타일이 안 먹지?"의 첫 진단.
fn dump(pass: &Pass) {
    for (tag, style) in &pass.styles {
        println!("{tag}: gap={:?} padding={:?} bg={:?}", style.gap, style.padding, style.bg);
    }
    println!("labels: {:?}", pass.labeled);
    println!("inputs: {}", pass.inputs);
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<Panel>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_gpui::ElmView as View;

    /// 헤드리스: 상태 계약 (어댑터 없이).
    #[test]
    fn state_contract() {
        let mut app = elm_magic::mount!(Panel);
        app.type_into("input", "elm");
        app.click_sel(&elm_magic::sel!(role Button));
        app.assert_text("ok 1");
    }

    /// 실제 창에서 `Pass`를 읽는 자리 — `#[gpui_kit::test]`(예제 17)로 실행한다.
    #[allow(dead_code)]
    fn adapter_reported_what_it_drew(view: &View<Panel>) -> (usize, usize, usize) {
        let pass = view.pass();
        dump(pass);
        (pass.styles.len(), pass.labeled.len(), pass.inputs)
    }
}