//! egui 예제 14 — `<Raw>` 탈출구 (elm-magic에 없는 egui 위젯 끼우기)
//!
//! **언제 쓰나**: 하이퍼링크, 이미지, 커스텀 페인트, egui 생태계 위젯처럼
//! elm-magic 태그에 없는 것을 한 자리에 넣어야 할 때. **유일한 escape hatch**다.
//!
//! **규칙** (사양서 7.3)
//! - `<Raw>|ui: &mut egui::Ui| { … }</Raw>` — 클로저가 **플랫폼 페이로드**를 받는다.
//!   egui 어댑터는 `&mut egui::Ui`, gpui 어댑터는 `&mut gpui_kit::Window`를 넘긴다.
//! - 헤드리스 테스트에서는 **무시된다**(`render_tree()`에 `[raw]`로만 보인다).
//! - `elm_magic::raw::invoke(tree, &mut payload)`로 어댑터가 하는 일을 테스트할 수 있다.
//!
//! **베스트 패턴**
//! - `<Raw>`는 **잎(leaf)으로만** 쓴다 — 안에서 elm 상태를 읽지 말고, 필요한 값은
//!   클로저가 캡처하게 한다(`move`).
//! - Raw를 쓰면 그 자리는 헤드리스 계약에서 빠진다. 가능하면 Raw 하나당
//!   "무엇이 그려져야 하는가"를 별도 테스트로 못 박는다(예: 위젯에 `ElementId`를 주고
//!   egui 입력으로 클릭).
//! - 생태계 위젯이 커지면 어댑터에 정식 태그를 추가하는 편이 낫다(`Widget` 트레이트).
//!
//! **주의**: 페이로드 타입이 어댑터와 다르면 다운캐스트에서 panic한다 — Raw는
//! **플랫폼 종속**이다(같은 트리를 egui와 gpui에 동시에 쓸 수 없다).

use elm_magic::prelude::*;

elm_magic::view! {
    fn Mixed(url = String::from("https://example.com")) {
        <Col>
            <Raw>|ui: &mut egui::Ui| {
                ui.hyperlink_to("docs", "https://example.com");
            }</Raw>
            "plain text"
            <Raw>|ui: &mut egui::Ui| {
                // egui 생태계 위젯도 그대로: 상태는 egui 쪽이 소유한다.
                ui.separator();
            }</Raw>
        </Col>
    }
}

/// Raw의 페이로드 타입을 **직접 만들 수 있는** 형태 — 어댑터 없이도 계약을 검증한다.
elm_magic::view! {
    fn Probe() {
        <Col>
            <Raw>|out: &mut String| { out.push_str("painted"); }</Raw>
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_is_invisible_to_the_headless_contract() {
        let app = elm_magic::mount!(Mixed);
        app.expect_text("plain text");
        assert!(app.render_tree().contains("[raw]"));
        assert!(!app.text().contains("docs")); // Raw는 텍스트 계약에 없다
    }

    #[test]
    fn raw_receives_the_platform_payload() {
        let app = elm_magic::mount!(Probe);
        let mut out = String::new();
        // 어댑터가 하는 일을 그대로 흉내 낸다(egui 어댑터는 &mut egui::Ui를 넘긴다).
        elm_magic::raw::invoke(app.element(), &mut out);
        assert_eq!(out, "painted");
    }
}

