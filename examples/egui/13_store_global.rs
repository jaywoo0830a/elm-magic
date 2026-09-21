//! egui 예제 13 — `#[store]` 전역 상태 (테마/세션처럼 화면 전체가 공유하는 것)
//!
//! **언제 쓰나**: 헤더·사이드바·푸터가 같은 값을 보고, 어느 컴포넌트에서 바꿔도
//! 전부 다시 그려져야 할 때. props로 내려보내는 "prop drilling"을 없앤다.
//!
//! **규칙** (`tests/store.rs`)
//! - `#[store] struct App { .. }`는 **컴포넌트보다 위**(같은 파일 상단)에 선언한다.
//! - 본문에서 `app.count`처럼 **구조체 이름의 소문자 접근자**로 읽고 쓴다.
//!   쓰기는 `app.count += 1` / `app.dark = !app.dark`.
//! - `#[store]`는 rustc가 구조체로 파싱하므로 **필드 기본값 문법을 쓸 수 없다**
//!   (`count: i32 = 0` 금지). 기본값이 필요하면 `store_fn!`을 쓴다.
//! - 필드마다 버전 카운터가 있어, 전역 쓰기가 프레임을 다시 돈다.
//!
//! **베스트 패턴**
//! - 전역에는 **"진실"만** 둔다(`dark: bool`). 파생 값(색, 문자열)은 렌더에서 계산한다.
//! - 마운트 때 초기화가 필요하면 `store_fn!`의 기본값으로 표현한다.
//! - unmount 시점에 바꿔야 하는 것은 전역이어야 한다(예제 12의 주의).
//!
//! **주의**: 전역 상태는 테스트 간에 공유된다 — 프로세스(테스트 바이너리) 안에서
//! 같은 `#[store]` 이름을 쓰면 서로 영향을 준다.

use elm_magic::prelude::*;

// 기본값이 필요 없으면 `#[store]`
#[store]
struct App {
    dark: bool,
    closed: i32,
}

// 기본값이 필요하면 `store_fn!`
elm_magic::store_fn! {
    Settings { level: i32 = 3, muted: bool = true }
}

elm_magic::view! {
    fn Header() {
        <Row>
            "dark: {app.dark}"
            <Button on_click={app.dark = !app.dark}>"theme"</Button>
        </Row>
    }
}

elm_magic::view! {
    fn Footer() {
        <Text>"footer sees dark: {app.dark}"</Text>
    }
}

elm_magic::view! {
    fn Session() {
        // 지역 슬롯은 unmount 시점에 죽어 있으므로 전역을 갱신한다.
        on_unmount { app.closed += 1 }
        <Text>"session open"</Text>
    }
}

elm_magic::view! {
    fn Root(show = true) {
        <Col>
            <Header />
            <Footer />
            "level: {settings.level}"
            "closed: {app.closed}"
            <If when={show}>
                <Session />
            <Else>
                <Text>"hidden"</Text>
            </Else>
            </If>
            <Button on_click={show = !show}>"toggle"</Button>
        </Col>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_is_shared_and_unmount_updates_it() {
        let mut app = elm_magic::mount!(Root);
        app.assert_text("footer sees dark: false");
        app.click("theme");
        app.assert_text("dark: true");
        app.assert_text("footer sees dark: true");

        app.assert_text("level: 3"); // store_fn! 기본값
        app.click("toggle");
        app.assert_text("closed: 1"); // on_unmount → 전역
    }
}
