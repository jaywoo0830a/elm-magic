//! egui 예제 08 — 입력 + 디바운스 검색 (폼의 기본형)
//!
//! **언제 쓰나**: 텍스트 입력이 곧바로 비싼 계산/네트워크로 이어질 때.
//!
//! **도구**
//! - `<Input value={query.clone()} on_change={query = _} on_enter={items.push(..)} />`
//!   — `_`는 이벤트가 준 값(입력이면 `String`).
//! - `on_change(query) after 300ms { results <- search(query.clone()) }`
//!   — **값이 바뀔 때마다 타이머가 리셋**된다(진짜 디바운스). `after`가 없으면 즉시.
//! - `on_change(query) { .. }` — 값이 그대로면 다시 실행되지 않는다.
//!
//! **베스트 패턴**
//! 1. 입력 슬롯(`query`)과 결과 슬롯(`results`)을 **분리**한다 — 입력은 즉시 반응하고
//!    결과만 늦게 온다(체감 지연 최소).
//! 2. 디바운스 본문은 `<-`(효과)로 두고 테스트에서 `mock!`으로 갈아 끼운다.
//! 3. `on_enter`는 "즉시 실행" 경로로 남겨 둔다 — 느린 디바운스를 기다리지 않게.
//!
//! **주의**: `_`는 이벤트 본문에서만 의미가 있다. 렌더 식에 쓰면 컴파일 에러다.

use elm_magic::prelude::*;

async fn search_api(query: String) -> Vec<String> {
    // 실제 구현: 네트워크. 테스트에서는 목으로 대체한다(예제 18).
    vec![format!("hit:{query}")]
}

elm_magic::view! {
    fn Search(
        query = String::new(),
        results: Vec<String> = vec![],
        pinned: Vec<String> = vec![],
    ) {
        on_change(query) after 300ms { results <- search_api(query.clone()) }
        <Col>
            <Input
                value={query.clone()}
                on_change={query = _}
                on_enter={pinned.push(query.clone())} />
            {results.map(|r| <Row class="result">"{r}"</Row>)}
            {pinned.map(|p| <Row class="pinned">"★ {p}"</Row>)}
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_waits_for_the_clock() {
        let app = elm_magic::mount!(Search);
        let mut app = app.mock(search_api, |q: String| vec![format!("hit:{q}")]);

        app.type_into("input", "rust");
        app.advance(299);
        app.assert_hidden("hit:rust");     // 아직
        app.advance(1);
        app.flush();
        app.assert_text("hit:rust");       // 300ms 도달
    }

    #[test]
    fn enter_pins_immediately() {
        let mut app = elm_magic::mount!(Search);
        app.type_into("input", "elm");
        app.press_enter();
        app.assert_text("★ elm");          // 디바운스를 기다리지 않는다
    }
}
