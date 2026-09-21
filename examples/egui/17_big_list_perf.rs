//! egui 예제 17 — 큰 목록 성능 (규칙 수 · 상태 추적 · 앱 경로 선택)
//!
//! **언제 쓰나**: 목록이 수백~수천 행이거나 프레임 예산(16.6ms)에 쫓길 때.
//!
//! **어디가 비싼가** (`crates/elm-magic-egui/benchmark/README.md` 실측)
//! - 어댑터 비용은 대부분 **노드마다 만드는 egui `Ui` 조작 수**로 귀속된다.
//! - `resolved_style_in`은 **매칭 규칙 수에 선형**이다 → 규칙을 늘리면 모든 노드가 느려진다.
//! - `:hover`/`:active`/`:focus` 규칙이 **하나도 없으면** 어댑터가 노드당 상태 추적
//!   (temp data 읽기/쓰기 + 컨텍스트 락)을 통째로 건너뛴다 — `style::needs_pointer_state()`.
//!   상태를 쓰지 않는 목록에서는 CSS에 `:hover`를 넣지 않는 것이 곧 최적화다.
//! - 참조 화면(1,000행 = 3,005노드)에서 `render` → `render_fast`가 한 프레임을
//!   4.02ms → 2.78ms로 줄였다(Pass 기록의 memcpy 제거).
//!
//! **베스트 패턴 (효과 순)**
//! 1. 앱 루프는 `render_fast`(예제 02).
//! 2. 상태가 없는 행은 `<For>`로 위치 기반(키 슬롯 비용 절약), 상태가 있으면 `key`.
//! 3. 셀렉터를 넓게 잡지 말고 클래스로 좁힌다 — 규칙 수 자체가 비용이다.
//! 4. 포인터 상태가 필요 없으면 `:hover` 규칙을 만들지 않는다.
//! 5. 그래도 부족하면 그 목록만 `<Raw>` + egui `ScrollArea::show_rows`(가상 스크롤)로
//!    넘긴다 — elm-magic은 "행이 화면에 몇 개 보이는가"를 알 필요가 없다.
//!
//! **주의**: 최적화는 **측정 먼저**. `bash ./bench`(저장소 루트)가 단계별 표를 찍는다.

use elm_magic::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct Line {
    id: usize,
    text: String,
}

elm_magic::view! {
    fn BigList(lines: Vec<Line> = vec![], selected = 0) {
        <Col class="list">
            <For each={lines} as={l}>
                <Row class="list__row">
                    "{l.text}"
                    <Button on_click={selected = l.id.clone()}>"pick"</Button>
                </Row>
            </For>
            "selected: {selected}"
        </Col>
    }
}

/// 가상 스크롤이 필요할 때의 탈출구 — 보이는 행만 그린다.
elm_magic::view! {
    fn VirtualList(lines: Vec<Line> = vec![]) {
        <Raw>|ui: &mut egui::Ui| {
            let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
            egui::ScrollArea::vertical().show_rows(ui, row_h, lines.len(), |ui, range| {
                for i in range {
                    ui.label(format!("{}: {}", lines[i].id, lines[i].text));
                }
            });
        }</Raw>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_list_is_a_pure_projection() {
        let lines: Vec<Line> = (0..500)
            .map(|i| Line { id: i, text: format!("row {i}") })
            .collect();
        let app = elm_magic::mount_with::<BigList>(BigListProps {
            lines: Some(lines),
            ..Default::default()
        });
        app.assert_text("row 0");
        app.assert_text("row 499");
    }

    /// 상태 추적 게이트가 켜졌는지 — CSS에 상태 셀렉터가 없으면 `false`여야 한다.
    #[test]
    fn pointer_state_is_off_when_no_state_selectors_exist() {
        // 이 테스트 바이너리에는 `:hover` 규칙이 없다.
        assert!(!elm_magic::style::needs_pointer_state());
    }
}
