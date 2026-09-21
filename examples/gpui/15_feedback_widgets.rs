//! gpui 예제 15 — 피드백 위젯이 **gpui-kit 컴포넌트**로 그려진다
//!
//! **언제 쓰나**: 로딩/진행/체크/배너/모달을 테마에 맞게, 애니메이션까지 살려서
//! 보여주고 싶을 때. 어댑터가 gpui-kit 컴포넌트를 골라 쓰기 때문에 **추가 코드가 없다**.
//!
//! **매핑** (`crates/elm-magic-gpui/src/lib.rs`)
//! | elm 태그 | gpui 쪽 |
//! |---|---|
//! | `<Check>` | gpui-kit `Checkbox` (테마·포커스 링 포함) |
//! | `<Progress value={0..1}>` | gpui-kit `Progress` (어댑터가 0..100으로 변환, `fill` 색 사용) |
//! | `<Spinner />` | gpui-kit `Spinner` (애니메이션) |
//! | `<Banner kind="..">` | 토큰 색 배경 + 패딩 (종류별 기본색: error/warn/success/info) |
//! | `<Modal title="..">` | 제목 + 자식 패널 (오버레이는 플랫폼 몫) |
//! | `<Divider />` | 테두리색 1px |
//! | `<Tab active={..}>` / `<Th>` | 클릭 가능한 텍스트 (활성 표시는 스타일로) |
//!
//! **핵심**
//! - `text-transform`은 `Text`뿐 아니라 `Button`/`Tab`/`Th`/`Banner`/`Checkbox` **라벨에도**
//!   적용된다(`transformed()`).
//! - `fill` 토큰은 `<Progress>`/`<Spinner>`의 악센트 색으로 쓰인다.
//! - `<Modal>`의 오버레이·포커스 트랩은 제공하지 않는다 — 필요하면 gpui-kit의
//!   `Modal`/`Dialog`를 `<Raw>`로 얹거나, 화면 전환(라우팅)으로 푼다.
//!
//! **주의**: gpui에 대응이 없는 속성은 조용히 건너뛴다(`letter-spacing`, `z-index`,
//! `mono`, `rotate`, `scale`, `pointer-events`). 레이아웃을 이들에 의존시키지 말 것.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::css! {
    .panel { gap: 8; padding: 16; bg: surface; radius: 8; }
    .progress { fill: success; }
    .label { text-transform: uppercase; font-size: 12; color: text_dim; }
}

elm_magic::view! {
    fn Panel(
        agreed = false,
        synced = false,
        pct = 0.25,
        tab = 0,
        open = false,
    ) {
        <Col class="panel">
            <Text class="label">"status"</Text>
            <Spinner />
            <Progress class="progress" value={pct} />
            <Divider />
            <Banner kind="error">"boom"</Banner>
            <Banner kind="success">"saved"</Banner>
            <Check checked={agreed} on_change={agreed = !agreed}>"동의"</Check>
            <Check checked={synced} on_change={synced = _}>"동기화"</Check>
            <Row>
                <Tab active={tab == 0} on_click={tab = 0}>"Home"</Tab>
                <Tab active={tab == 1} on_click={tab = 1}>"Stats"</Tab>
            </Row>
            <Row>
                <Th on_click={tab = 1}>"Name"</Th>
                <Td>"cell"</Td>
            </Row>
            <Button on_click={open = !open}>"모달 열기"</Button>
            <Modal title="확인" on_close={open = false}>
                <Text>"본문"</Text>
                <Button on_click={open = false}>"닫기"</Button>
            </Modal>
        </Col>
    }
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

    #[test]
    fn widget_states_round_trip() {
        let mut app = elm_magic::mount!(Panel);
        app.toggle("동의");
        assert!(app.render_tree().contains("Check \"동의\" checked=true"));
        app.set_check("동기화", true);
        assert!(app.render_tree().contains("Check \"동기화\" checked=true"));
    }

    #[test]
    fn text_transform_is_resolved_in_the_core() {
        // 스타일 해석은 플랫폼 독립이다 — 어댑터는 라벨에 적용만 한다.
        let app = elm_magic::mount!(Panel);
        let label = app
            .element()
            .children()
            .and_then(|c| c.iter().find(|e| e.tag() == "text"))
            .expect("Text 자식");
        let style = label.resolved_style(&elm_magic::style::Palette::dark());
        assert!(style.transform.is_some(), "uppercase가 해석돼야 한다");
    }
}