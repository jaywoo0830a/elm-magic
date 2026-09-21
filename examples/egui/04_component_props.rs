//! egui 예제 04 — 컴포넌트 props (필수 / 기본값 / `mount_with` 대응)
//!
//! **언제 쓰나**: 컴포넌트를 부모 상태에서 분리해 재사용할 때.
//!
//! **규칙** (`crates/elm-magic-macros/src/view.rs`)
//! - `view!`의 **매개변수 = 상태 슬롯 + props**. 매크로가 `XProps` 구조체와
//!   `impl Component for X`를 만들어 준다.
//! - 모든 props 필드는 `Option<T>`다. `None`이면 **선언한 기본값**을 쓴다.
//! - 기본값이 없는 매개변수는 **필수 prop** — `mount_with!`/`frame`에서 `None`이면
//!   "필수 prop이 없습니다" panic으로 즉시 알려준다.
//! - 타입은 추론 가능하면 생략해도 된다(`n = 0` → `i32`, `"x"` → `String`).
//!   컬렉션은 `Vec<Todo> = vec![]`처럼 **주석을 달아야** 한다.
//!
//! **베스트 패턴**: egui 앱 경로와 헤드리스 테스트 경로가 **같은 props**를 쓰게
//! `XProps`를 한 번 만들고 양쪽에 넘긴다.

use elm_magic::prelude::*;

elm_magic::view! {
    // `name`은 필수 prop, `count`는 기본값 0.
    fn Greeting(name: String, count = 0) {
        <Col>
            "Hello, {name}"
            "count: {count}"
            <Button on_click={count += 1}>"greet"</Button>
        </Col>
    }
}

/// 앱: props를 **한 번만** 만들어 프레임마다 재사용한다.
fn app_frame(ui: &mut egui::Ui, ctx: &mut Ctx) {
    let props = GreetingProps {
        name: Some("elm".to_string()),
        count: None, // 기본값(0) 사용 — 명시하려면 Some(0)
        ..Default::default() // children까지 포함해 나머지를 채운다
    };
    let tree = elm_magic::frame::<Greeting>(ctx, &props);
    elm_magic_egui::render_fast(ui, &tree, &mut ctx.arena);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 같은 props로 헤드리스 검증 — egui 없이 계약을 먼저 못 박는다(예제 18).
    #[test]
    fn greeting_contract() {
        let mut app = elm_magic::mount_with::<Greeting>(GreetingProps {
            name: Some("elm".to_string()),
            ..Default::default()
        });
        app.expect_text("Hello, elm");
        app.click("greet");
        app.expect_text("count: 1");
    }
}
