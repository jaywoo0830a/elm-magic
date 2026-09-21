//! gpui 예제 07 — 제어 흐름 태그 (`<If>` / `<For>` / `<Switch>` / `<>`)
//!
//! **언제 쓰나**: 화면 본문에서 조건/반복을 태그로 쓰고 싶을 때. **어댑터가 바뀌지
//! 않는 이유**: 제어 흐름 태그는 매크로가 **컴파일타임에 펼치는** 문법이다 —
//! 트리에는 감싼 요소만 남으므로 egui/gpui 어댑터는 존재조차 모른다.
//!
//! **계약** (`tests/0.8/*.rs`)
//! - `<If when={..}>` + `<Else>` — 둘 다 레이아웃 노드를 만들지 않는다.
//! - `<For each={..} as={t} key={t.id}>` — key가 있으면 keyed 슬롯(자식 상태 보존).
//! - `<Switch on={..}>` + `<Case when={패턴}>` + `<Default>` — 패턴 바인딩 허용.
//! - `<> … </>` — 프래그먼트(`tag() == ""`).
//! - `{items}`처럼 `IntoView` 값을 중괄호에 바로 넣어도 된다(0.8): `Vec<T>`/`Option<T>`/
//!   이터레이터가 펼쳐진다.
//!
//! **gpui 특이점**
//! - `Overflow`/`display: none`/`visibility: hidden`은 어댑터가 gpui 쪽으로 옮긴다 —
//!   `display: none`이면 어댑터가 **빈 `div()`를 반환**해 자리도 차지하지 않는다.
//! - `letter-spacing`/`z-index`/`mono`/`rotate`/`scale`/`pointer-events`는 gpui에 대응이
//!   없어 건너뛴다(문서화된 예외) — 이 속성에 레이아웃을 의존하지 말 것.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

#[derive(Clone, PartialEq, Debug)]
enum Filter { All, Active, Done }

#[derive(Clone, PartialEq, Debug)]
enum Status { Idle, Loading, Failed(String) }

elm_magic::view! {
    fn TodoPanel(
        items: Vec<String> = vec![],
        filter: Filter = Filter::All,
        status: Status = Status::Idle,
    ) {
        <Col>
            <If when={items.is_empty()}>
                <Text class="muted">"항목 없음"</Text>
            </If>

            <For each={items} as={t} key={t}>
                <Row>"{t}"</Row>
            </For>

            <Switch on={status}>
                <Case when={Status::Idle}><Text>"대기"</Text></Case>
                <Case when={Status::Loading}><Spinner /></Case>
                <Case when={Status::Failed(e)}><Banner kind="error">{e}</Banner></Case>
                <Default><Text>"—"</Text></Default>
            </Switch>

            <>
                <Divider />
                <Text class="muted">"{filter:?}"</Text>
            </>
        </Col>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(cx.new(ElmView::<TodoPanel>::new))
    }
}