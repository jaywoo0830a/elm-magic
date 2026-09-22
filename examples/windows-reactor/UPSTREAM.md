# 업스트림 windows-rs 샘플과의 대조표

이 디렉터리의 예제 30개를 **[microsoft/windows-rs `crates/samples`](https://github.com/microsoft/windows-rs/tree/master/crates/samples)**
(특히 `reactor/` 55개 디렉터리)와 하나씩 맞대어 본 결과다. 예제 헤더의 사실 주장은
**`windows-reactor 0.100.0` 소스(배포본)**와 `cargo check`/`cargo test`로 검증했다.

## 0. 기준이 되는 버전 (중요)

| | 값 | 근거 |
|---|---|---|
| 이 저장소의 `windows-reactor` | **0.100.0** | `examples/windows-reactor/Cargo.toml`, `Cargo.lock` |
| crates.io 최신 | 0.100.0 | `crates.io/api/v1/crates/windows-reactor` (max_stable = 0.100.0) |
| 업스트림 `crates/libs/reactor` | 0.100.0 (마스터 HEAD) | `crates/libs/reactor/Cargo.toml` |
| 업스트림 샘플의 의존 | `windows-reactor = { workspace = true }` → `path = "crates/libs/reactor"` | 저장소 루트 `Cargo.toml` |

즉 **업스트림 샘플은 배포본이 아니라 저장소 소스(마스터)를 따라간다.** 그래서 마스터
샘플에는 배포본 0.100.0에 **없는** API가 섞여 있다 — 확인된 예: `message-box`의
`context.run_window`(0.100.0에는 `ComponentContext::{open_window, sender,
spawn_background, spawn_background_with_rejection, window}`만 있다). 이 저장소는
배포본 기준이므로 그런 API는 **쓰지 않는다**(해당 예제는 대안을 적어 두었다).

## 1. 업스트림 관용구 — 이 저장소가 따라야 할 것

| 업스트림 | 이 저장소 |
|---|---|
| 샘플마다 **독립 패키지 디렉터리**(`reactor/<이름>/Cargo.toml` + `src/main.rs`) | `examples/windows-reactor/` 하나의 패키지 + `[[example]]` 등록(같은 취지, 저장소 구조상 이렇게) |
| `#![windows_subsystem = "windows"]` (콘솔 숨김) | 예제는 **콘솔을 남긴다**(panic/로그 확인용) — 예제 01 헤더에 한 줄로 안내 |
| `use windows_reactor::*;` | 필요한 이름을 **명시적으로** import(무엇을 쓰는지 보이게 — 예제라는 목적) |
| `fn main() { App::run_component::<C>(()).unwrap(); }` | 같다(에러 문구만 한국어) |
| `context.window_title(..)` · `window_visuals(..)` · `on_window_size(..)` | 예제 01 · 12 · 20이 호스트 컴포넌트의 `view()`에서 선언 |
| `context.spawn_background(..)` (async) | 예제 11(호스트) · 12(티커) — `ElmView`의 `ElmMessage`는 `Rc`라 `Send`가 아니다 |
| `ElementRef::new()` + `.element_ref(&r)` + `request_focus()` | 어댑터가 만든 컨트롤에는 참조를 줄 수 없다 — 예제 05 헤더에 이유를 적었다 |
| `KeyAccelerators::new([..])` + `AcceleratorKey` | 예제 05(Enter) · 12(Ctrl+R) · 20(Ctrl+R → 새 창) |
| `context.open_window(view)` | 예제 20(런타임) · `App::run_windows([..])`(시작 시) |
| `KeyedView::new(키, 뷰)` + `keyed_children` | 어댑터는 **위치 기반 키**를 넘긴다(문서화된 한계) — 예제 07 |
| `Context::new(..)` + `View::provide` + `use_context` | `#[store]`와 층이 다르다는 설명 — 예제 13 |
| 스타일 계층 없음(WinUI 테마/리소스) | 어댑터도 `css!`를 지원하지 않는다(의도) — 예제 16 · 21~30 |

## 2. 대응표 (이 저장소 30개 → 업스트림)

| 이 예제 | 업스트림에서 같이 볼 것 | 업스트림에서 배울 것 |
|---|---|---|
| 01 최소 카운터 | `counter` `controlled` `self_contained` `startup_perf` | 창 하나 띄우는 최소 골격, 부트스트랩 비용 |
| 02 props | `component-input` `function-component` | `Input: Clone + PartialEq`와 `input_changed` |
| 03 여러 뷰 | `gallery` `navigation` | 한 창에 여러 컴포넌트를 붙이는 구성 |
| 04 이벤트/메시지 | `function-component` `radio-buttons` | 메시지 하나 = 상태 갱신 하나 |
| 05 입력 | `controlled` `form` `text-box-border` `text-trimming` | `TextBox`가 controlled, `ElementRef`로 포커스 |
| 06 제어 흐름 태그 | (대응 없음 — elm 매크로 고유) | Reactor에는 `if`/`for`가 Rust 문법이다 |
| 07 목록과 키 | `keyed-list-reorder` `virtual` `scroll-viewer` | 진짜 키로 행 상태 보존, `ItemsRepeater` 가상화 |
| 08 `<Raw>` | `element-ref` `drag-drop` `pointer-*` `tooltip-placement` | WinUI 컨트롤에 직접 접근하는 자리 |
| 09 비동기 효과 | `async-state` `use-effect` | `spawn_background` + 결과 메시지 |
| 10 스트림 | `async-state` `gallery` | 진행률을 값마다 반영하는 형태 |
| 11 호스트 상태/콜백 | `component-input` `async-state` `message-box` | props 내려보내기 + 콜백 prop 올려보내기 |
| 12 수명/단축키/시계 | `use-effect` `window` `notifyicon` | effect key, `on_window_size` |
| 13 `#[store]` | `context` | Reactor `Context` + `View::provide` |
| 14 피드백 위젯 | `form` `radio-buttons` `message-box` | `ProgressBar`/`InfoBar`/`CheckBox` |
| 15 모달 | `message-box` `exit-transition` | `ContentDialog` · 전환 |
| 16 스타일 없이 | `theme-brush` `color-scheme` `theme-transition` `shape` `card` `text-block` | 테마 브러시/리소스가 색·모양을 담당 |
| 17 탭과 헤더 | `tab-view-add-button` `navigation-view-icons` `navigation-view-pane` | `TabView`·`NavigationView` 조립 |
| 18 셸 레이아웃 | `navigation` `responsive-navigation` `notepad` | 사이드바/본문 셸 구성 |
| 19 로딩/에러/빈 | `async-state` `form` | 비동기 3분기와 재시도 동선 |
| 20 앱 골격 | `window` `secondary-window` `self_contained` `lightweight-resources` | 창 선언·다중 창·배포 형태 |
| 21~25 토큰·타이포·간격·모양·색 | `theme-brush` `color-scheme` `shape` `card` `text-block` `icon` `icon-elements` | WinUI 리소스 사전과 값 체계 |
| 26 버튼 변형 | `button-icon` `app-bar-icon` `radio-buttons` | 아이콘 버튼/변형 |
| 27 Grid 레이아웃 | `stacker` `gallery` `solitaire` `minesweeper` | `Grid`의 행/열 정의 |
| 28 스크롤 | `scroll-viewer` `virtual` `gallery` | `ScrollViewer`/`ItemsRepeater` |
| 29 상태와 전환 | `opacity-transition` `scale-transition` `exit-transition` `theme-transition` | 전환은 값이 바뀔 때 WinUI가 보간 |
| 30 종합 스타일 가이드 | `gallery` | 여러 주제를 한 앱으로 묶기 |
| (해당 없음) | `tictactoe` `dotsweeper` `solitaire` `calculator` `notepad` `stacker` | 완성 앱 크기의 조립 예 — 이 저장소에는 없다 |

## 3. 비교에서 드러난 사실 — 이번에 예제를 고친 이유

1. **호스트는 `ElmView` 인스턴스에 접근할 수 없다.** `ElementRef<T>`는 `T:
   ReferenceControl`(컨트롤 전용)이라 컴포넌트 참조를 얻을 길이 없다. 그래서 *개정 전*
   예제 12/20의 `fn tick(view: &ElmView<..>, ..)` 헬퍼는 **앱에서 호출할 수 없는
   코드**였다(컴파일은 됐지만 실행 경로가 없다). → 호스트가 시계·단축키를 소유하는
   형태로 고쳤다.
2. **`drive::run`은 시계를 밀지 않는다.** `ElmView::update`가 부르는 것은 `drive::run`
   하나이므로 `<- f() after …`는 앱에서 due가 되지 않는다. `on_tick`은 더 나아가
   `drive::run_at`으로도 돌지 않는다(슬롯 시계를 미는 것은 테스트 하네스의 `advance`
   뿐이다). → 예제 12가 이 계약을 **테스트로 고정**했다.
3. **키 이벤트는 WinUI 가속기로 잡는다.** `on_key` 핸들러는 `Ctx::keys`(아레나 안)에
   있고 호스트가 아레나를 만질 수 없다. `AcceleratorKey`(0.100.0)는 `R` · `Enter` ·
   사칙연산 · 숫자패드만 안다 — `Ctrl+S` 같은 문자 키는 없다. → 예제 05(Enter) ·
   12/20(Ctrl+R)이 `KeyAccelerators`를 쓴다.
4. **비동기는 호스트가 맡는다.** `ElmMessage`가 `Rc<dyn Fn(..)>`를 나르므로 `Send`가
   아니고, 따라서 elm 안에서 `spawn_background`를 쓸 수 없다. 업스트림 `form`/
   `async-state`의 자리는 **호스트 컴포넌트**다(예제 11 · 12).
5. **스타일은 WinUI의 몫이다.** 업스트림에는 elm-magic의 `css!` 같은 계층이 없다 —
   이 어댑터가 스타일 계층을 지원하지 않기로 한 결정과 같은 결론이다(예제 16 · 21~30).
6. **키가 필요한 목록은 업스트림이 더 낫다.** `KeyedView::new(키, ..)`로 행 상태를
   보존한다 — elm 트리는 키를 노출하지 않아 어댑터가 위치 기반 키를 쓴다(예제 07의
   문서화된 한계, `<Raw>` 탈출구).

## 4. 이번 개정에서 실제로 바꾼 것

- **예제 01** — 창을 선언하는 최소 호스트(`window_title`/`window_visuals`) 추가,
  업스트림 대응·`#![windows_subsystem]` 안내.
- **예제 05** — "Enter 불가"를 **실제 해법**으로 교체: 호스트가 값을 소유하고
  `KeyAccelerators(Enter)`로 제출하는 두 번째 예(`QuickHost`/`QuickAdd`) + 테스트.
- **예제 07 · 13 · 15** — 업스트림 대응(`keyed-list-reorder`/`virtual`, `context`,
  `message-box`)과 확인된 API 목록 추가.
- **예제 12** — "누가 구동하는가"를 표로 정정하고, 죽은 `tick(..)` 헬퍼를
  `spawn_background` 자기-재무장 티커 + `KeyAccelerators(Ctrl+R)` + `window_visuals`로
  교체. `drive` 모듈은 **테스트/커스텀 호스트용**임을 명시하고 계약 테스트를 추가.
- **예제 19** — 상태 3분기(`Ready`/`Empty`/`Failed`)를 **효과로 실제 생성**
  (`phase = Loading; phase <- load(..)`) — 업스트림 `async-state`의 자리를 elm 문법으로.
- **예제 20** — 앱 골격을 실행 가능하게: `window_visuals` · `on_window_size` ·
  `open_window`(Ctrl+R → 새 창) · 정정된 구동 표.
- **전체** — `cargo check --examples --features winui` 경고 0개(테스트 전용 계약은
  `#[allow(dead_code)]` + 이유 주석으로 표시).

## 5. 어댑터로 넘길 후속 제안 (예제 밖)

- `ElmView::update`가 **시계를 밀도록** 한다: `Instant` 기반으로
  `drive::run_at(ctx, elapsed)`을 부르면 `<- f() after ..`가 앱에서도 돈다
  (`on_tick`은 슬롯 시계가 따로라 코어 쪽 손질이 필요하다).
- `plan` 층이 **`on_key` 핸들러를 `KeyAccelerators`로 매핑**하면 단축키가 elm 문법
  그대로 동작한다(`ElmMessage`에 `Key(String)` 변형 추가). 지금은 호스트가 우회한다.
- **진짜 키**를 트리에 노출해 `KeyedView::new`에 넘기면 업스트림 `keyed-list-reorder`와
  같은 수준이 된다(현재는 위치 기반).

## 6. 업스트림을 직접 읽을 때

```
crates/samples/reactor/<이름>/src/main.rs   ← 샘플 본문 (창 하나 = 파일 하나)
crates/libs/reactor/src/                    ← 라이브러리 (public API는 src/core/public.rs 등)
```

```powershell
# 이 저장소의 예제 30개 — 목록/테스트/실행
cd examples\windows-reactor
powershell -ExecutionPolicy Bypass -File run-tests.ps1 -List
powershell -ExecutionPolicy Bypass -File run-tests.ps1
.\run-example.ps1 20
```