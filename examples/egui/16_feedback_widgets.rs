//! egui 예제 16 — 피드백 위젯 (Banner / Spinner / Progress / Modal / Divider …)
//!
//! **언제 쓰나**: "지금 무슨 일이 일어나는 중인가"를 보여주는 기본 부품들.
//! elm-magic이 태그 17종을 제공하므로 이 대부분은 **Raw 없이** 해결된다.
//!
//! **태그 → 그리는 것**
//! | 태그 | 인자 | 비고 |
//! |---|---|---|
//! | `<Banner kind="error">` | `kind`: `error`/`warn`/`success`/`info` | 색을 안 정하면 종류별 토큰 기본색 |
//! | `<Spinner />` | — | 로딩 표시(헤드리스 텍스트는 `[spinner]`) |
//! | `<Progress value={0.5} />` | `f64` 0..1 | elm은 0..1, gpui-kit은 0..100으로 어댑터가 변환 |
//! | `<Divider />` | — | 구분선 |
//! | `<Strong>` | 텍스트 | 강조 |
//! | `<Check checked={..} on_change={..}>` | bool 이벤트는 `_` | `toggle("라벨")`로 테스트 |
//! | `<Modal title=".." on_close={..}>` | children | 닫기는 **핸들러로만** (오버레이는 플랫폼 몫) |
//! | `<Tab active={..} on_click={..}>` | — | `<Th>`(정렬 가능 헤더) / `<Td>`와 함께 표에 쓴다 |
//!
//! **베스트 패턴**
//! - 로딩/에러/빈 상태는 **3분기**로 명시한다(예제 19) — `<Spinner>` 하나만 띄우면
//!   사용자는 멈춘 건지 모른다.
//! - `Modal`은 `open` 슬롯을 두고 `on_close`에서 닫는다. 열림 상태는 **부모가** 갖는다.
//! - 표는 `<Row>` + `<Th>`/`<Td>`로 만든다 — 정렬은 `Th on_click`에서 목록을 다시 정렬.
//!
//! **주의**: `<Check>`의 `on_change`는 **bool 값 이벤트**라 `checked = _`가 자연스럽다.
//! `<Input>`은 `String`이다(예제 08).

use elm_magic::prelude::*;

elm_magic::view! {
    fn Panel(
        open = false,
        synced = false,
        tab = 0,
        title = String::new(),
        items: Vec<String> = vec![],
    ) {
        <Col>
            <Spinner />
            <Divider />
            <Strong>"Total: 3"</Strong>
            <Banner kind="error">"boom"</Banner>
            <Banner kind="success">"saved"</Banner>
            <Progress value={0.5} />
            <Check checked={open} on_change={open = !open}>"agree"</Check>
            <Check checked={synced} on_change={synced = _}>"sync"</Check>

            <Row>
                <Tab active={tab == 0} on_click={tab = 0}>"Home"</Tab>
                <Tab active={tab == 1} on_click={tab = 1}>"Stats"</Tab>
            </Row>
            <Row>
                <Th on_click={items.sort()}>"Name"</Th>
                <Td>"cell"</Td>
            </Row>

            <Modal title="확인" on_close={open = false}>
                <Text>"modal body"</Text>
                <Button on_click={open = false}>"close"</Button>
            </Modal>
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_and_modal_dispatch() {
        let mut app = elm_magic::mount!(Panel);
        assert!(app.render_tree().contains("Check \"agree\" checked=false"));
        app.toggle("agree");
        assert!(app.render_tree().contains("Check \"agree\" checked=true"));

        app.click("close"); // Modal의 자식 버튼도 클릭된다
        assert!(app.render_tree().contains("Check \"agree\" checked=false"));
    }

    #[test]
    fn tab_switch_is_exclusive() {
        let mut app = elm_magic::mount!(Panel);
        app.click("Stats");
        let tree = app.render_tree();
        assert!(tree.contains("Tab \"Stats\" active"));
        assert!(!tree.contains("Tab \"Home\" active"));
    }
}
