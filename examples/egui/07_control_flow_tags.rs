//! egui 예제 07 — 제어 흐름 태그: `<If>` / `<For>` / `<Switch>` / `<>`
//!
//! **언제 쓰나**: 0.8부터 화면 본문에서 조건/반복을 **태그로** 쓴다. 매크로가
//! "레이아웃 노드 없는 분기"로 펼치므로 트리 모양이 더 이상 조건 때문에 흔들리지 않는다.
//!
//! **계약** (`tests/0.8/*.rs`가 고정)
//! - `<If when={..}>` … `<Else>` … `</If>` — `Else`가 없으면 false일 때 아무것도 안 그린다.
//! - `<For each={..} as={t} key={t.id}>` — `each`는 `IntoIterator`, `as`는 아이템 바인딩.
//! - `<Switch on={..}>` + `<Case when={패턴}>` + `<Default>` — **패턴 매칭**이라
//!   `Status::Failed(e)`처럼 바인딩도 된다.
//! - `<> … </>` — 프래그먼트: 레이아웃 없이 묶는다(`tag()`는 `""`).
//! - 제어 흐름 태그는 **자식 수를 늘리지 않는다** — 감싼 요소가 그대로 자식이 된다.
//!
//! **기존 문법도 그대로 산다**: `{if …}`, `{items.map(…)}`. 추가 중심이므로
//! 마이그레이션 압박이 없다(`tests/0.8/interop.rs`가 등가성을 고정).
//!
//! **주의**: `<Case>`/`<Default>`는 `<Switch>` 안에서만, `<Else>`는 `<If>` 안에서만 쓴다.

use elm_magic::prelude::*;

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
            // 1) 조건 — Else 생략 가능
            <If when={items.is_empty()}>
                <Text class="muted">"항목 없음"</Text>
            </If>

            // 2) 반복 — key가 없으면 위치 기반
            <For each={items} as={t}>
                <Row>"{t}"</Row>
            </For>

            // 3) 다분기 — match의 UI 버전 (패턴 바인딩 허용)
            <Switch on={status}>
                <Case when={Status::Idle}>
                    <Text>"대기"</Text>
                </Case>
                <Case when={Status::Loading}>
                    <Spinner />
                </Case>
                <Case when={Status::Failed(e)}>
                    <Banner kind="error">{e}</Banner>
                </Case>
                <Default>
                    <Text>"—"</Text>
                </Default>
            </Switch>

            // 4) 프래그먼트 — 레이아웃 없이 묶기
            <>
                <Text>"{filter:?}"</Text>
                <Divider />
            </>
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_flow_is_transparent_to_layout() {
        let app = elm_magic::mount_with::<TodoPanel>(TodoPanelProps {
            items: Some(vec!["a".into(), "b".into()]),
            status: Some(Status::Failed("boom".into())),
            ..Default::default()
        });
        let children = app.element().children().expect("Col 자식");
        // "a", "b" 2 + Switch 결과 1 + Fragment 결과 2 = 5 (If는 false라 0)
        assert_eq!(children.len(), 5, "{}", app.render_tree());
        app.assert_text("boom");
    }
}
