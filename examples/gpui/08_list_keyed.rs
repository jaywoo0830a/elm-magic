//! gpui 예제 08 — 목록과 keyed 슬롯 (재정렬에도 자식 상태가 따라가게)
//!
//! **언제 쓰나**: 행마다 상태(카운터, 편집 중 텍스트, 선택)를 가진 목록.
//! gpui든 egui든 **규칙은 같다** — keyed 슬롯은 elm-magic 코어 기능이라 어댑터와 무관하다.
//!
//! **규칙** (사양서 9.5, `tests/store.rs`)
//! - `key`가 있으면 키가 자식 상태를 따라간다(재정렬/삽입에도 유지).
//! - 키가 사라지면 슬롯이 버려지고 `on_unmount`가 돈다.
//! - 키가 없으면 위치 기반 — 상태 없는 행에만 쓴다.
//!
//! **문장 위치의 메서드 호출은 슬롯 변경으로 변환된다** (`items.push(x)`,
//! `items.remove(요소)`, `items.reverse()`). 즉 `Vec`의 **실제 메서드**여야 한다 —
//! 없는 메서드를 쓰면 컴파일 에러다(조용히 무시되지 않는다).
//!
//! **gpui 특이점**
//! - `<For>`는 레이아웃 노드를 만들지 않으므로 gpui 트리에서는 행들이 부모
//!   `flex_col` 아래에 **평평하게** 들어간다.
//! - elm-magic에는 가상 스크롤이 없다. 수천 행이 필요하면 `<Raw>`로 gpui
//!   `Window`를 받아 gpui-kit의 리스트 컴포넌트를 쓰되, **선택/스크롤 상태는
//!   elm 슬롯**에 두어 계약(헤드리스 테스트)을 유지한다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

#[derive(Clone, PartialEq, Debug)]
struct Item {
    id: u64,
    text: String,
}

/// 행마다 자기 상태(`n`)를 갖는다 — keyed 슬롯 덕분에 재정렬에도 따라간다.
elm_magic::view! {
    fn RowItem(text: String, n = 0) {
        <Row>
            "{text}"
            "n: {n}"
            <Button on_click={n += 1}>"count"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn List(items: Vec<Item> = vec![], reversed = false, hide_none = false) {
        <Col>
            <Check checked={hide_none} on_change={hide_none = !hide_none}>"메모"</Check>
            <For each={items} as={it} key={it.id}>
                <RowItem key={it.id} text={it.text.clone()} />
            </For>
            "reversed: {reversed}"
            <Button on_click={items.reverse(), reversed = !reversed}>"reverse"</Button>
            <Button on_click={items.remove(0)}>"pop"</Button>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<List>::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> elm_magic::testing::TestApp<List> {
        elm_magic::mount_with::<List>(ListProps {
            items: Some(vec![
                Item { id: 1, text: "a".into() },
                Item { id: 2, text: "b".into() },
            ]),
            ..Default::default()
        })
    }

    /// 정렬을 뒤집어도 행 상태(n)가 항목을 따라간다.
    #[test]
    fn row_state_follows_the_key() {
        let mut app = list();
        app.click("count"); // 첫 행(a)의 카운터 → n: 1
        app.assert_text("n: 1");

        app.click("reverse");
        let text = app.text();
        let b = text.find('b').expect("b");
        let n1 = text.find("n: 1").expect("n: 1");
        let a = text.find('a').expect("a");
        assert!(b < n1, "reverse 후 b가 먼저: {text:?}");
        assert!(a < n1, "'n: 1'은 여전히 a와 함께 있다: {text:?}");
    }

    #[test]
    fn removal_drops_the_slot() {
        let mut app = list();
        app.click("pop"); // 첫 요소 삭제
        let text = app.text();
        assert!(!text.contains('a'), "삭제된 행이 남았다: {text:?}");
    }
}