//! windows-reactor 예제 12 — 수명/단축키/시계를 Reactor에서 구동하기
//! **업스트림 대응**: `use-effect`(수명/effect key) · `window`(`on_window_size`) ·
//! `notifyicon`(주기 갱신). 업스트림에는 타이머 API가 없어서 **시계는 호스트가** 공급한다 —
//! 아래 표가 그 계약을 고정한다.
//!
//!
//! **언제 쓰나**: 붙을 때 로드, 떨어질 때 정리, 단축키, 주기 갱신.
//!
//! **elm 쪽 문법** (플랫폼과 무관, `tests/lifecycle.rs`·`tests/timers.rs`)
//! - `on_mount { .. }` / `on_unmount { .. }` — 인스턴스 수명에 한 번씩.
//!   unmount 시점의 **지역 슬롯 쓰기는 버려진다** → 전역(`#[store]`)만 갱신한다.
//! - `on_key("Ctrl+S") { .. }` — 핸들러는 `Ctx::keys`에 (키, 핸들러)로 쌓인다.
//! - `on_tick(500ms) { .. }` — 슬롯의 시계가 주기를 넘길 때마다 본문이 돈다.
//!
//! **Reactor 앱에서 누가 구동하나** (windows-reactor 0.100.0 실측)
//!
//! | elm 문법 | WinUI 앱 경로 | 이유 |
//! |---|---|---|
//! | `<- f()` (지연 없음) | **돈다** | `ElmView::update`가 메시지 처리 뒤 같은 발행에서 `drive::run`을 부른다 |
//! | `on_mount` / `on_unmount` | **돈다** | 프레임이 슬롯 수명을 관리한다 |
//! | `<- f() after …` | **돌지 않는다** | `drive::run`은 **시계를 밀지 않는다**(`set_now`가 없다). 밀려면 `drive::run_at`을 불러야 하는데 **호스트가 아레나에 접근할 수 없다** — `ElementRef<T>`는 `T: ReferenceControl`(컨트롤 전용)이라 컴포넌트 참조를 얻을 길이 없다 |
//! | `on_tick(..)` | **돌지 않는다** | 게다가 `drive::run_at`으로도 안 된다 — `on_tick`의 슬롯 시계는 테스트 하네스의 `advance`가 미는 별도 시계다(아래 `drive_run_leaves...` 테스트가 그 사실을 고정한다). 그래서 **시간이 흐르는 일은 호스트가 소유**하고 결과를 props로 내린다(아래 `schedule_tick`) |
//! | `on_key(..)` | **지원 키만 돈다** | 어댑터가 마지막 프레임의 `Ctx::keys`를 WinUI **`KeyAccelerators`**로 매핑해서 루트에 붙인다(업스트림 `calculator`의 자리). `AcceleratorKey`(0.100.0)가 아는 키는 `R`·`Enter`·사칙연산·숫자패드 + `Ctrl`뿐이라 **`Ctrl+S`는 매핑되지 않는다** — 그런 키는 호스트가 직접 붙인다(예제 05의 두 번째 예) |
//!
//! **그럼 `drive` 모듈은 누구를 위한 것인가**: 아레나를 **직접 소유한 쪽**이다 —
//! 헤드리스 테스트(`flush`/`advance`/`press_key`가 정확히 이 경로다)와, 자기 `Ctx`로
//! 구동하는 커스텀 호스트. `ElmView`를 Reactor에 맡긴 앱에서는 부를 수 없다(위 표의
//! 이유가 그것이다). 이 파일의 마지막 테스트가 그 계약(`drive::run`은 시계를 밀지
//! 않는다)을 고정한다.
//!
//! **베스트 패턴**: 시간이 흐르는 일(주기 갱신·애니메이션·폴링)은 **호스트가 소유**하고
//! elm에는 **값만** 내려보낸다. `on_key`는 어댑터가 매핑해 주므로 elm에 그대로 두면
//! 앱에서도 돈다(지원 키 한정). `on_tick`은 헤드리스 계약으로 남겨 두면(테스트가
//! 지킨다) 화면 코드는 플랫폼을 모른 채 그대로 재사용된다(예제 20).

use elm_magic_windows_reactor::{ElmInput, ElmView};
use windows_reactor::{App, Component, ComponentContext, View, ViewContext, WindowVisuals};

elm_magic::view! {
    fn Editor(text = String::new()) {
        // 지원 키 — 어댑터가 `AcceleratorKey::R` + `Ctrl` 가속기로 옮긴다.
        on_key("Ctrl+R") { text = String::from("reloaded") }
        // 지원하지 않는 키 — `AcceleratorKey`에 `S`가 없다. 핸들러는 아레나에 남지만
        // 앱에서는 불리지 않는다(호스트가 직접 붙여야 하는 자리).
        on_key("Ctrl+S") { text = String::from("saved") }
        <Text>"{text}"</Text>
    }
}

elm_magic::view! {
    fn Clock(now = 0) {
        on_tick(500ms) { now += 1 }
        <Text>"tick: {now}"</Text>
    }
}

elm_magic::view! {
    fn Profile(id = 1, user = String::new()) {
        on_mount { user <- api_user(id) }
        <Text>"user: {user}"</Text>
    }
}

async fn api_user(id: i32) -> String {
    format!("user-{id}")
}

/// 호스트 — **시계를 자기 것으로 소유한다**(elm은 props로 본다).
///
/// `ElmView` 인스턴스에 접근할 수 없으므로(`ElementRef<T>`는 컨트롤 전용) 주기 작업은
/// 여기서 돈다: 백그라운드에서 잠깐 자고 틱 메시지를 보내고, `update`가 **자기 자신을
/// 다시 예약**한다. 단축키는 반대로 **elm이 선언한 것을 어댑터가 내려보낸다**
/// (`on_key("Ctrl+R")` → WinUI `KeyAccelerators`) — 호스트가 붙일 필요가 없다.
struct Root {
    ticks: u64,
}

#[derive(Clone)]
enum RootMessage {
    /// 호스트 티커 — 자기 재무장.
    Tick,
}

impl Component for Root {
    type Input = ();
    type Message = RootMessage;

    fn create(_input: &(), context: &ComponentContext<Self>) -> Self {
        schedule_tick(context);
        Self { ticks: 0 }
    }

    fn update(&mut self, message: RootMessage, context: &ComponentContext<Self>) {
        match message {
            RootMessage::Tick => {
                self.ticks += 1;
                // 자기 재무장 — 0.100.0에는 타이머 API가 없어 이렇게 주기를 만든다.
                // 실제 앱에서는 **멈추는 조건**을 둔다(창이 닫히면 재무장하지 않는다).
                schedule_tick(context);
            }
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 수명/단축키/시계");
        // 창 크기는 호스트가 정한다 (업스트림 `window` 샘플과 같은 자리).
        // (elm만으로 끝내려면 이 선언을 `ElmInput::window_title/visuals`로 옮긴다 — 예제 01.)
        context.window_visuals(WindowVisuals::new().client_size(560.0, 420.0));

        View::fragment((
            View::component::<ElmView<Editor>>(ElmInput::new(EditorProps::default())),
            View::component::<ElmView<Clock>>(ElmInput::new(ClockProps::default())),
            View::component::<ElmView<Profile>>(ElmInput::new(ProfileProps::default())),
            // 호스트가 소유한 값 — elm 문법으로 표현할 수 없는 것들은 이렇게 내려보낸다.
            format!("host ticks: {}", self.ticks),
        ))
    }
}

/// 주기 작업 하나를 예약한다 — **호스트가 소유**한다(`Message: Send` 필요).
///
/// `spawn_background`는 Windows 스레드 풀에서 `work`를 돌리고 반환값을 메시지 큐에
/// 넣는다(업스트림 `async-state`·`form` 샘플이 쓰는 바로 그 API).
fn schedule_tick(context: &ComponentContext<Root>) {
    let _ = context.spawn_background(|_cancellation| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        RootMessage::Tick
    });
}

fn main() {
    App::run_component::<Root>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use elm_magic::prelude::*;
    use super::*;
    use elm_magic_windows_reactor::{drive, plan};

    /// elm의 키 선언은 마지막 프레임이 **아레나에 남긴다** — 어댑터의
    /// `attach_key_accelerators`가 그 목록을 읽어 매핑 가능한 것만 WinUI 가속기로
    /// 내려보내고, 나머지(`Ctrl+S`)는 호스트 몫으로 남긴다.
    #[test]
    fn shortcuts_are_registered_in_the_arena() {
        let mut ctx = Ctx::new();
        let props = EditorProps::default();
        let _ = elm_magic::frame::<Editor>(&mut ctx, &props);

        let registered: Vec<&str> = ctx.keys.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(
            registered,
            vec!["Ctrl+R", "Ctrl+S"],
            "선언 순서대로 쌓인다(어댑터가 매핑할 목록이 곧 이것이다)"
        );

        assert!(drive::dispatch_key(&mut ctx, "Ctrl+R"), "on_key가 등록돼야 한다");
        assert!(drive::dispatch_key(&mut ctx, "Ctrl+S"), "등록은 되지만 매핑은 안 된다");
        assert!(!drive::dispatch_key(&mut ctx, "Ctrl+Q"), "없는 키는 false");

        let tree = elm_magic::frame::<Editor>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("saved"), "마지막으로 돈 키가 이긴다");
    }

    #[test]
    fn tick_respects_the_period() {
        let mut app = elm_magic::mount!(Clock);
        app.advance(500);
        app.assert_text("tick: 1");
        app.advance(200); // 주기가 덜 지났다
        app.assert_text("tick: 1");
        app.advance(300);
        app.assert_text("tick: 2");
    }

    #[test]
    fn mount_effect_runs_when_driven() {
        let mut ctx = Ctx::new();
        let props = ProfileProps::default();
        let _ = elm_magic::frame::<Profile>(&mut ctx, &props);
        assert!(drive::run(&mut ctx), "on_mount 효과가 실행돼야 한다");
        let tree = elm_magic::frame::<Profile>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("user: user-1"));
    }

    /// 계약: `ElmView::update`가 부르는 것은 `drive::run`뿐이고 **시계는 밀리지 않는다**.
    /// `drive::run_at`은 아레나의 `after`용 시계만 밀어서 `on_tick`은 여전히 돌지 않는다 —
    /// 슬롯 시계를 미는 것은 테스트 하네스의 `advance`뿐이다(그래서 앱에서는 시간이
    /// 흐르는 일을 호스트가 소유한다 · 헤더의 표).
    ///
    /// 이 테스트가 깨지면 어댑터/코어의 시계 계약이 바뀐 것이므로 예제와 헤더를 함께
    /// 고쳐야 한다.
    #[test]
    fn drive_run_leaves_the_clock_alone_so_on_tick_never_fires() {
        let mut ctx = Ctx::new();
        let props = ClockProps::default();
        let _ = elm_magic::frame::<Clock>(&mut ctx, &props);

        assert!(!drive::run(&mut ctx), "시계가 0이면 due가 아니다");
        assert!(
            !drive::run_at(&mut ctx, 500),
            "`run_at`은 `after`용 시계만 민다 — `on_tick`은 그대로다"
        );
        let tree = elm_magic::frame::<Clock>(&mut ctx, &props);
        assert!(plan(&tree).1.has_text("tick: 0"), "앱 경로에서는 on_tick이 안 돈다");

        // 테스트 하네스만이 슬롯 시계를 민다 — 이 차이가 헤더의 안내 이유다.
        let mut app = elm_magic::mount!(Clock);
        app.advance(500);
        app.assert_text("tick: 1");
    }
}
