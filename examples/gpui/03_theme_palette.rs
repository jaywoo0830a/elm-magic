//! gpui 예제 03 — 테마 = 팔레트 (`ActiveTheme` → `Token`)
//!
//! **언제 쓰나**: 라이트/다크/커스텀 테마가 `css!` 색에 그대로 반영되길 원할 때.
//! gpui 쪽에서는 **아무것도 하지 않아도** 그렇게 동작한다 — 이 예제는 그 규칙과
//! 확인 방법을 고정한다.
//!
//! **매핑** (`crates/elm-magic-gpui/src/lib.rs::palette`)
//! | `css!` 토큰 | gpui-kit `ThemeColor` |
//! |---|---|
//! | `primary` / `on_primary` | `primary` / `primary_foreground` |
//! | `surface` / `surface_alt` | `popover` / `muted` (shadcn 관례) |
//! | `background` | `background` |
//! | `text` / `text_dim` | `foreground` / `muted_foreground` |
//! | `error` / `warn` / `success` / `info` | `danger` / `warning` / `success` / `info` |
//! | `border` / `overlay` | `border` / `overlay` |
//! | `shadow` | `foreground`의 10% 알파 |
//!
//! **핵심**
//! - 팔레트는 **매 렌더마다** `cx.theme()`에서 다시 만든다(캐시된 전역이 아니다).
//!   테마를 바꾸면 다음 프레임부터 `css!` 색이 함께 바뀐다.
//! - 그래서 "테마 토큰으로만 색을 쓴" 앱은 테마 전환 코드가 **0줄**이다.
//! - 반대로 특정 색을 고정하고 싶으면 `css!`에 그 토큰을 쓰지 말고 테마를 바꾼다
//!   (어댑터가 임의 색을 넣는 API는 없다 — 팔레트가 유일한 통로다).
//!
//! **주의**: `gpui_kit::init(cx)` 없이 창을 열면 `ActiveTheme`가 기본값이라
//! "다크인데 흰 배경" 같은 증상이 난다(예제 01의 주의).

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::css! {
    .card { padding: 12; gap: 6; bg: surface; radius: 8; }
    .card__title { color: text; font-size: 16; weight: bold; }
    .hint { color: text_dim; font-size: 12; }
    .danger { bg: error; color: on_primary; padding: 4 8; radius: 4; }
}

elm_magic::view! {
    fn Card(title = String::new()) {
        <Col class="card">
            <Text class="card__title">"{title}"</Text>
            <Text class="hint">"테마 토큰은 ActiveTheme에서 온다"</Text>
            {children}
        </Col>
    }
}

elm_magic::view! {
    fn ThemeDemo() {
        <Col>
            <Card title="Surface">
                <Text class="danger">"error 토큰"</Text>
            </Card>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 참고: 여기서 `cx.theme()`이 곧 컬러의 출처다.
        //   let theme = cx.theme();
        //   let palette = elm_magic_gpui::palette(theme);
        div().child(cx.new(ElmView::<ThemeDemo>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::TestAppContext;

    /// 매핑이 살아 있는지 테스트로 고정한다 (테마가 바뀌면 여기가 먼저 깨진다).
    #[gpui_kit::test]
    fn palette_follows_the_active_theme(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(|cx| {
            let theme = cx.theme();
            let palette = elm_magic_gpui::palette(theme);

            // 어댑터 내부의 `color_of`와 같은 변환 (그 함수는 비공개라 여기서 인라인한다).
            let expected = {
                let rgb = theme.colors.popover.to_rgb();
                let u = |v: f32| (v * 255.0).round() as u8;
                elm_magic::style::Color::rgba(u(rgb.r), u(rgb.g), u(rgb.b), u(rgb.a))
            };
            assert_eq!(
                palette.get(elm_magic::style::Token::Surface),
                expected,
                "`bg: surface`는 theme.popover여야 한다"
            );
        });
    }
}