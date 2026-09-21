//! egui 예제 19 — 로딩/에러/빈 상태 3분기 (화면이 "멈춘 것처럼" 보이지 않게)
//!
//! **언제 쓰나**: 비동기 결과를 기다리는 모든 화면. `<Spinner>`만 띄우면 사용자는
//! 실패한 건지 느린 건지 알 수 없다.
//!
//! **모델**: 상태를 enum 하나로 만들고 `<Switch>`로 **빠짐없이** 분기한다.
//! `Result`를 그대로 쓰기보다 `Idle/Loading/Ready/Empty/Failed`처럼 UI가 알아야 할
//! 구분을 명시하는 편이 낫다 — "빈 목록"과 "아직 안 옴"은 다른 화면이다.
//!
//! **베스트 패턴**
//! - 분기마다 **복구 행동**을 하나씩 둔다(Retry / 다시 시도 / 입력하러 가기).
//! - 실패 메시지는 `Banner kind="error"`로 눈에 띄게, 재시도 버튼은 바로 아래에.
//! - 빈 상태는 안내 문구 + 다음 행동을 함께 보여준다(`Text class="muted"`).
//! - 로딩 중에도 **이전 내용을 지우지 않는다**(가능하면) — 화면 깜빡임이 줄어든다.
//!
//! **주의**: `<-` 효과의 결과는 슬롯에 **값**으로 들어간다. 실패를 표현하려면
//! 슬롯 타입을 `Result<_, String>`로 두고 `<-`가 그 값을 돌려주게 한다.

use elm_magic::prelude::*;

#[derive(Clone, PartialEq, Debug)]
enum Phase {
    Idle,
    Loading,
    Ready(Vec<String>),
    Empty,
    Failed(String),
}

async fn load_items() -> Result<Vec<String>, String> {
    Ok(vec!["alpha".into(), "beta".into()])
}

elm_magic::view! {
    fn ItemsView(phase: Phase = Phase::Idle) {
        <Col>
            <Switch on={phase}>
                <Case when={Phase::Idle}>
                    <Button on_click={phase = Phase::Loading}>"불러오기"</Button>
                </Case>
                <Case when={Phase::Loading}>
                    <Row>
                        <Spinner />
                        "불러오는 중…"
                    </Row>
                </Case>
                <Case when={Phase::Ready(items)}>
                    <For each={items} as={it}>
                        <Row>"{it}"</Row>
                    </For>
                </Case>
                <Case when={Phase::Empty}>
                    <Text class="muted">"항목이 없습니다 — 새로 추가해 보세요"</Text>
                </Case>
                <Case when={Phase::Failed(msg)}>
                    <Col>
                        <Banner kind="error">"{msg}"</Banner>
                        <Button on_click={phase = Phase::Loading}>"다시 시도"</Button>
                    </Col>
                </Case>
            </Switch>
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_phase_has_a_visible_branch() {
        let app = elm_magic::mount!(ItemsView);
        app.assert_text("불러오기");

        let loading = elm_magic::mount_with::<ItemsView>(ItemsViewProps {
            phase: Some(Phase::Loading),
            ..Default::default()
        });
        loading.assert_text("[spinner]");

        let empty = elm_magic::mount_with::<ItemsView>(ItemsViewProps {
            phase: Some(Phase::Empty),
            ..Default::default()
        });
        empty.assert_text("항목이 없습니다");

        let failed = elm_magic::mount_with::<ItemsView>(ItemsViewProps {
            phase: Some(Phase::Failed("timeout".into())),
            ..Default::default()
        });
        failed.assert_text("timeout");
        failed.assert_text("다시 시도");
    }

    #[test]
    fn retry_returns_to_loading() {
        let mut app = elm_magic::mount_with::<ItemsView>(ItemsViewProps {
            phase: Some(Phase::Failed("timeout".into())),
            ..Default::default()
        });
        app.click("다시 시도");
        app.assert_text("[spinner]");
    }
}
