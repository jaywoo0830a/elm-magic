//! egui 예제 05 — children + 콜백 prop (부모-자식 통신)
//!
//! **언제 쓰나**: 재사용 가능한 "껍데기" 컴포넌트(`Card`/`Panel`)에 자식을 넣고,
//! 자식이 부모에게 값을 올려보낼 때.
//!
//! **핵심**
//! - `children`은 **매크로가 항상 넣어 주는 prop**이다. 본문에서 `{children}`을
//!   쓰면 그 인스턴스의 자식이 그대로 들어간다(안 쓰면 그냥 무시된다).
//! - 콜백 prop은 `fn(T)` 타입으로 선언하고, 자식 쪽에서 `on_pick(item.id)`처럼
//!   함수처럼 부른다. 부모는 `on_pick={picked = _}`로 **슬롯 대입에 연결**한다.
//! - 자식에 넘길 때는 `key={..}`를 붙인다 — 목록이 재정렬돼도 자식 상태가 따라간다.
//!
//! **주의**: 콜백 prop을 지정하지 않으면 **조용한 no-op**이다(테스트에 유리).
//! 필수로 만들고 싶으면 기본값 없이 선언해 `None`이면 panic하게 둔다.

use elm_magic::prelude::*;

#[derive(Clone, PartialEq)]
struct Item {
    id: i32,
    name: String,
}

/// 자식: 값을 위로 올린다. `on_pick`은 콜백 prop(타입 `fn(i32)`).
elm_magic::view! {
    fn ItemRow(item: Item, on_pick: fn(i32)) {
        <Row on_click={on_pick(item.id)}>
            "{item.name}"
        </Row>
    }
}

/// 껍데기: `children`을 받는다.
elm_magic::view! {
    fn Card(title = String::new()) {
        <Col class="card">
            "card: {title}"
            {children}
        </Col>
    }
}

/// 부모: 상태(`picked`)를 자식 콜백에 연결한다.
elm_magic::view! {
    fn Shell(items: Vec<Item> = vec![], picked = 0) {
        <Col>
            <Card title="shell">
                {items.map(|i| <ItemRow key={i.id} item={i} on_pick={picked = _} />)}
            </Card>
            "picked: {picked}"
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell() -> elm_magic::testing::TestApp<Shell> {
        elm_magic::mount_with::<Shell>(ShellProps {
            items: Some(vec![
                Item { id: 1, name: "one".into() },
                Item { id: 2, name: "two".into() },
            ]),
            ..Default::default()
        })
    }

    #[test]
    fn child_raises_value_to_parent() {
        let mut app = shell();
        app.assert_text("card: shell");   // children이 껍데기 안에 들어갔다
        app.click("two");
        app.assert_text("picked: 2");     // 자식 → 부모 상태
    }
}
