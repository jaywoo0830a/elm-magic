//! gpui 예제 16 — `css!` 없이 스타일 등록 (`style::register` 직접 호출)
//!
//! **언제 쓰나**: 매크로를 쓰고 싶지 않을 때, 스타일을 **런타임 값**에서 만들 때,
//! 또는 시작 섹션(ctor)이 돌지 않는 환경(wasm 등)에서 스타일이 사라지는 문제를
//! 피하고 싶을 때.
//!
//! **사실 관계** (`crates/elm-magic-macros/src/css.rs`)
//! - `css!`가 하는 일은 **시작 섹션에서 `style::register(vec![(셀렉터, StyleSpec)])`를
//!   대신 불러 주는 것**뿐이다. 스타일의 출처는 코어의 전역 레지스트리 하나다.
//! - 그래서 `register`를 직접 부르면 **완전히 동일하게** 동작한다. 리포지토리 안에도
//!   그렇게 쓰는 코드가 있다(`crates/elm-magic-egui/benchmark/rules.rs`,
//!   `benchmark/state.rs`).
//! - gpui 어댑터는 `css!`를 참조하지 않는다 — `resolved_style_in(..)`만 본다.
//!   즉 **`css!` 없는 앱이 정상 경로**다(`crates/elm-magic-gpui/tests/adapter.rs`에는
//!   `css!`가 한 줄도 없다).
//!
//! **잃는 것 (트레이드오프)**
//! - 컴파일타임 검증: 잘못된 셀렉터는 **런타임 panic**, 잘못된 값/토큰 이름은 타입
//!   오류로만 잡힌다(`StyleSpec` 필드가 `pub`이라 오타 필드는 컴파일 에러).
//! - `init_styles()`는 `css!`가 **기록해 둔** 항목만 되살린다 — 직접 등록한 것은
//!   자기 함수를 다시 부르면 된다(오히려 명시적이라 안전하다).
//!
//! **규칙**
//! - 같은 셀렉터는 **먼저 등록된 것이 이긴다**(멱등). 캐스케이드는
//!   "명시도 → 등록 순서"이므로 **호출 순서**가 곧 우선순위다.
//! - `:hover`/`:active`/`:focus` 규칙을 등록하면 `style::needs_pointer_state()`가
//!   `true`가 되어 어댑터가 상태 추적을 켠다(비용 발생).
//! - 등록은 **렌더 전에** 끝내야 한다. 프레임 중간에 등록하면 형제 노드가 다른
//!   규칙 집합을 본다.

use elm_magic::prelude::*;
use elm_magic::style::{Align, Direction, Edges, Len, Shadow, StyleSpec, Token};
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

/// `main()`에서 **한 번** 부른다 (앱 시작 = 스타일 설치 시점).
fn install_styles() {
    elm_magic::style::register(vec![
        (
            ".app",
            StyleSpec {
                width: Some(Len::Fill),
                height: Some(Len::Fill),
                bg: Some(Token::Background),
                ..StyleSpec::NONE
            },
        ),
        (
            ".card",
            StyleSpec {
                gap: Some(8.0),
                padding: Some(Edges::splat(16.0)),
                bg: Some(Token::Surface),
                radius: Some(8.0),
                shadow: Some(Shadow { dx: 0.0, dy: 2.0, blur: 8.0, spread: 0.0 }),
                shadow_color: Some(Token::Shadow),
                ..StyleSpec::NONE
            },
        ),
        // 상태 셀렉터 — 등록하면 포인터 상태 추적이 켜진다(성능 주의).
        (
            ".card:hover",
            StyleSpec { bg: Some(Token::SurfaceAlt), ..StyleSpec::NONE },
        ),
        // 태그 셀렉터도 같은 문법이다.
        (
            "button",
            StyleSpec { radius: Some(6.0), ..StyleSpec::NONE },
        ),
        // 후손/자식 셀렉터.
        (
            ".card .title",
            StyleSpec {
                color: Some(Token::Text),
                font_size: Some(16.0),
                bold: Some(true),
                ..StyleSpec::NONE
            },
        ),
    ]);
}

/// 런타임 값에서 스타일을 만든다 — `css!`로는 불가능한 지점.
fn install_dynamic(density: f32) {
    elm_magic::style::register(vec![(
        ".row",
        StyleSpec {
            gap: Some(4.0 * density),
            padding: Some(Edges::new(2.0 * density, 8.0 * density, 2.0 * density, 8.0 * density)),
            align: Some(Align::Center),
            direction: Some(Direction::Row),
            ..StyleSpec::NONE
        },
    )]);
}

elm_magic::view! {
    fn Card(title = String::new()) {
        <Col class="card">
            <Text class="title">"{title}"</Text>
            {children}
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<Card>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic::style::{lookup_class, Palette};

    #[test]
    fn register_is_equivalent_to_css_macro() {
        install_styles();
        let card = lookup_class("card").expect(".card 등록");
        assert_eq!(card.get("gap").as_deref(), Some("8"));
        assert_eq!(card.get("bg").as_deref(), Some("surface"));

        // 같은 셀렉터를 다시 등록해도 늘지 않는다(먼저 등록이 이긴다).
        let before = elm_magic::style::len();
        elm_magic::style::register(vec![(
            ".card",
            StyleSpec { gap: Some(999.0), ..StyleSpec::NONE },
        )]);
        assert_eq!(elm_magic::style::len(), before);
        assert_eq!(lookup_class("card").unwrap().get("gap").as_deref(), Some("8"));
    }

    #[test]
    fn resolution_matches_the_declaration() {
        install_styles();
        let app = elm_magic::mount!(Card);
        let col = app.element().resolved_style(&Palette::dark());
        assert_eq!(col.gap, Some(8.0));
        assert_eq!(col.padding, Some(Edges::splat(16.0)));
        assert_eq!(col.bg, Some(Palette::dark().get(Token::Surface)));
    }

    #[test]
    fn state_selectors_turn_on_pointer_tracking() {
        install_styles();
        assert!(
            elm_magic::style::needs_pointer_state(),
            "`:hover` 규칙을 등록했으니 어댑터가 상태를 읽는다"
        );
    }
}