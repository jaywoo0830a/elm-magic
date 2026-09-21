//! egui 예제 06 — 리스트와 keyed 슬롯 (상태가 항목을 따라가는 목록)
//!
//! **언제 쓰나**: 항목마다 자체 상태(선택 여부, 편집 중 텍스트, 카운터)를 가진
//! 목록. 재정렬/삭제/삽입이 일어나도 상태가 **항목을 따라가야** 할 때.
//!
//! **규칙** (사양서 9.5)
//! - `key`가 **있으면** keyed 슬롯: 키가 같으면 자식 상태가 보존되고, 키가 사라지면
//!   슬롯이 버려지며 `on_unmount`가 돈다(`tests/store.rs`).
//! - `key`가 **없으면** 위치 기반: 순서가 바뀌면 상태도 자리대로 섞인다.
//! - 키는 `key_of`(=`{:?}`)로 문자열화된다 — `Debug`만 있으면 어떤 타입이든 된다.
//!
//! **베스트 패턴**
//! - 리스트 항목에 상태가 하나라도 있으면 **항상 `key`**.
//! - 항목 클릭은 `<For each={..} as={r}>`의 아이템 바인딩(`r`)을 이벤트 본문에서
//!   그대로 쓴다 — 리스트를 빌린 참조를 이벤트가 캡처하는 문제(E0716)를 매크로가
//!   처리해 준다(`tests/syntax.rs`, `tests/0.8/for_loop.rs`).
//! - 삭제는 `Vec` 슈가(`items.remove(요소)`)로 쓴다(사양서 3.3, `tests/syntax.rs`).
//!
//! **주의**: `<For>`는 레이아웃 노드를 만들지 않는다 — 컨테이너 자식 수를 세는
//! 테스트가 래퍼 때문에 어긋나지 않는다(`tests/0.8/for_loop.rs`).

use elm_magic::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct Item {
    id: u64,
    text: String,
}

elm_magic::view! {
    fn List(items: Vec<Item> = vec![], selected = String::new()) {
        <Col>
            <For each={items} as={it} key={it.id}>
                <Row>
                    "{it.text}"
                    <Button on_click={selected = it.text.clone()}>"select"</Button>
                    <Button on_click={items.remove(it.clone())}>"remove"</Button>
                </Row>
            </For>
            "selected: {selected}"
            <Button on_click={items.reverse()}>"reverse"</Button>
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic::sel;

    fn list() -> elm_magic::testing::TestApp<List> {
        elm_magic::mount_with::<List>(ListProps {
            items: Some(vec![
                Item { id: 1, text: "a".into() },
                Item { id: 2, text: "b".into() },
            ]),
            ..Default::default()
        })
    }

    #[test]
    fn keyed_rows_survive_reorder() {
        let mut app = list();
        app.click_sel(&sel!(role Button, "select")); // 트리 순서 첫 일치 = "a"
        app.assert_text("selected: a");

        app.click("reverse"); // 순서만 뒤집힌다
        let text = app.text();
        assert!(text.find('b').unwrap() < text.find('a').unwrap(), "{text:?}");
        app.assert_text("selected: a"); // 선택 상태는 항목을 따라갔다
    }

    #[test]
    fn remove_by_element_drops_the_slot() {
        let mut app = list();
        app.click_sel(&sel!(role Button, "remove")); // 첫 행의 remove
        let text = app.text();
        assert!(!text.contains('a'), "삭제된 항목이 남아 있다: {text:?}");
        assert!(text.contains('b'));
    }
}
