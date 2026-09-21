//! windows-reactor 예제 07 — 목록과 **위치 기반 키** (문서화된 한계)
//!
//! **언제 쓰나**: 행마다 상태가 있는 목록.
//!
//! **elm 쪽 규칙** (사양서 9.5, `tests/store.rs`)
//! - `<For key={it.id}>`가 있으면 **elm 아레나**가 키별로 자식 상태를 보존한다
//!   (재정렬/삽입에도 유지). 키가 사라지면 슬롯이 버려지고 `on_unmount`가 돈다.
//! - 문장 위치의 메서드 호출은 슬롯 변경으로 변환된다: `items.push(x)`,
//!   `items.remove(요소)`, `items.reverse()`. **실제 `Vec` 메서드**여야 한다.
//!
//! **Reactor 쪽 한계 (중요)**
//! - elm 트리는 **key를 노출하지 않는다**(코어가 아레나 슬롯 경로로 관리한다).
//!   그래서 어댑터는 `keyed_children`을 쓰되 키를 **인덱스**로 만든다.
//! - 결과: **elm 상태**는 키를 따라가지만, **WinUI 컨트롤**은 위치로 재사용된다.
//!   재정렬 직후 네이티브 상태(예: 캐럿 위치, 스크롤)가 옛 위치에 남을 수 있다.
//!   → 재정렬이 잦고 네이티브 상태가 중요한 행은 **행 전체를 다시 만들거나**
//!     `<Raw>`로 `keyed_children`에 진짜 키를 넘겨야 한다.
//! - elm-magic에는 가상 스크롤이 없다. 큰 목록은 `<Raw>`로 `ItemsRepeater`/
//!   `ListView`를 직접 쓰고, **선택/스크롤 상태는 elm 슬롯**에 두어 계약을 유지한다.
//!
//! **베스트 패턴**: 행이 자기 상태를 가질 때만 `<For>`를 쓰고, 정적인 목록은
//! 그냥 `<Text>`를 나열한다(불필요한 슬롯을 만들지 않는다).

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext};

#[derive(Clone, PartialEq, Debug)]
struct Item {
    id: u64,
    text: String,
}

/// 행마다 자기 상태(`n`)를 갖는다 — elm keyed 슬롯이 재정렬에도 따라간다.
elm_magic::view! {
    fn RowItem(text: String, n = 0) {
        <Row>
            <Text>"{text}"</Text>
            "n: {n}"
            <Button on_click={n += 1}>"count"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn List(items: Vec<Item> = vec![], reversed = false) {
        <Col>
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

impl Component for Root {
    type Input = ();
    type Message = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 목록");
        View::component::<ElmView<List>>(ElmInput::new(ListProps {
            items: Some(vec![
                Item {
                    id: 1,
                    text: "a".to_string(),
                },
                Item {
                    id: 2,
                    text: "b".to_string(),
                },
            ]),
            ..Default::default()
        }))
    }
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> elm_magic::testing::TestApp<List> {
        elm_magic::mount_with::<List>(ListProps {
            items: Some(vec![
                Item {
                    id: 1,
                    text: "a".to_string(),
                },
                Item {
                    id: 2,
                    text: "b".to_string(),
                },
            ]),
            ..Default::default()
        })
    }

    /// elm 상태는 키를 따라간다 — 정렬을 뒤집어도 `n: 1`이 a 옆에 남는다.
    #[test]
    fn row_state_follows_the_key() {
        let mut app = list();
        app.click("count"); // 첫 행(a)의 카운터
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
        app.click("pop");
        assert!(!app.text().contains('a'), "삭제된 행이 남았다");
    }
}
