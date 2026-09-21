# 참고 예제 (컴파일 대상 아님)

`elm-magic`을 **egui**(`elm-magic-egui`), **gpui-kit**(`elm-magic-gpui`),
**WinUI 3 / windows-reactor**(`elm-magic-windows-reactor`)에서 각각 어떻게 쓰는지
정리한 베스트 패턴 모음이다. 예제 20개씩, 총 60개.

```
examples/
├── README.md               ← 지금 이 파일
├── egui/01_…20_*.rs        ← egui 어댑터 + eframe/egui 앱 패턴
├── gpui/01_…20_*.rs        ← gpui-kit 어댑터 + ElmView 패턴
└── windows-reactor/01_…20_*.rs  ← WinUI 3 어댑터 (스타일 계층 없음)
```

**windows-reactor 세트는 `css!`를 쓰지 않는다** — 그 어댑터는 스타일 계층을
**지원하지 않기로** 했다(의도). 색·간격·정렬은 WinUI 테마/리소스(`ThemeBrush`,
`Border`, `FontWeight`)가 담당하고, WinUI 수준 조정은 `<Raw>`가 정식 통로다.
자세한 이유는 `windows-reactor/16_styling_without_css.rs`와
`crates/elm-magic-windows-reactor/src/lib.rs`의 크레이트 문서에 있다.

## 왜 컴파일되지 않는가 (의도)

Cargo의 자동 타겟 탐색은 **`examples/*.rs`와 `examples/<name>/main.rs`만** 본다.
이 저장소의 예제는 `examples/egui/`, `examples/gpui/` **하위 디렉터리의 `*.rs`** 이고
`main.rs`라는 이름도 쓰지 않으므로 어떤 타겟으로도 등록되지 않는다:

```
$ cargo metadata --no-deps --format-version 1     # example 타겟 0개 (4개 패키지 모두)
$ cargo build --examples
warning: target filter `examples` specified, but no targets matched; this is a no-op
```

그래서 이 파일들은

- CI(`cargo test --workspace` / `--all-features` / 릴리스 모드)에 영향이 **없고**,
- `elm-magic-egui`가 의존하지 않는 `eframe`, `gpui-kit`의 부트스트랩 코드처럼
  워크스페이스에서 컴파일할 수 없는 조각도 마음껏 담을 수 있다.
- 대신 **타입 검사도 받지 않는다** — 버전이 올라가면 API가 어긋날 수 있다.
  실제로 돌리는 코드를 쓸 때는 테스트(`tests/`, `crates/*/tests/`)를 정본으로 삼는다.
  각 파일의 `#[cfg(test)] mod tests`는 **코어만 쓰므로** `tests/`에 붙여 넣으면
  실제로 실행된다(어댑터 호출 부분은 빼고).

`//!` 머리에 어떤 상황에 쓰는 패턴인지, 무엇을 조심해야 하는지 적어 두었다.
각 파일은 복사해서 앱에 붙이는 것을 전제로 한다.

## 예제를 실제로 돌리려면

컴파일 대상이 아니므로 "실행"하려면 앱 쪽에서 다음을 갖춰야 한다.

| 예제 | 추가로 필요한 것 |
|---|---|
| `egui/*` | `eframe = "0.36"`(egui 0.36과 같은 마이너), `elm-magic-egui` |
| `gpui/*` | `gpui-kit = "0.6"`(+ 테스트는 dev-dependencies에서 `features = ["test-support"]`) |
| `windows-reactor/*` | `windows-reactor = "0.100"` + `elm-magic-windows-reactor` (**Windows 전용**, WASDK 런타임 필요) |
| 공통 | `elm-magic` (워크스페이스 경로 의존이면 `path = "…"`) |

그리고 **앱 루프를 직접 만들어야 한다** — 아래 두 가지는 어댑터가 해 주지 않는다.

## 반드시 알아야 할 세 가지 (예제에서 가장 자주 걸리는 함정)

1. **`<-`(효과)와 `->`(스트림)는 앱이 구동해야 한다.**
   `frame`/`render`는 트리만 그린다. 효과는 `Arena::take_due()`, 스트림은
   `Arena::take_streams()`/`push_streams()`, 시계는 `Arena::set_now(ms)`가 필요하다.
   테스트의 `flush`/`advance`/`pump`가 하는 일이 정확히 이것이다.
   windows-reactor 어댑터는 이걸 `drive` 모듈로 공개하고, `ElmView::update`가
   자동으로 부른다 — 구현은 `egui/09`·`egui/10`·`gpui/11`·`gpui/13`·`gpui/20`·
   `windows-reactor/09`·`10`·`12`·`20`에 있다.
2. **`on_key`는 앱이 키를 디스패치해야 한다.**
   핸들러는 `Ctx::keys`(`Vec<(String, KeyHandler)>`)에 쌓인다 —
   테스트의 `press_key("Ctrl+S")`처럼 앱이 찾아 호출한다(`gpui/13`의 `dispatch_key`,
   `windows-reactor`의 `drive::dispatch_key`). `on_tick`도 같은 원리로 시계가 있어야 돈다.
3. **`#[store]`의 범위는 "아레나 하나"다.**
   `Arena.stores`에 있으므로 **한 컴포넌트 트리 안에서만** 공유된다 —
   별개의 `ElmView`/`Ctx`끼리는 공유하지 않는다(예제 `egui/13`, `gpui/14`,
   `windows-reactor/13`). 화면 사이 공유는 (1) 루트 컴포넌트로 묶기,
   (2) 플랫폼 상태를 props로 내려보내기 중 하나로 푼다.

## 세 어댑터의 공통 계약

egui/gpui는 **코어가 `ResolvedStyle`까지 해석**하고, 어댑터는 그 결과를 옮기기만 한다 —
그래서 `view!` + `css!` 컴포넌트는 두 플랫폼에서 그대로 재사용된다.
windows-reactor는 **스타일 계층을 옮기지 않는다**(의도) — 구조·상태·이벤트만 옮긴다.

| | egui | gpui-kit | windows-reactor |
|---|---|---|---|
| 진입점 | `elm_magic_egui::render(ui, &tree, &mut ctx.arena)` | `ElmView::<C>::new(cx)` (gpui `Render`) | `App::run_component::<ElmView<C>>(ElmInput::new(props))` |
| 앱 경로(빠름) | `render_fast` (기록 안 함) | `Pass`는 항상 채우지만 노드당 1회 push | 계획(plan)은 항상 계산 — 계획 층이 매핑의 정본 |
| 팔레트 출처 | `Visuals::dark_mode` 또는 인자 | `ActiveTheme`(gpui-kit 테마) | 없음 (WinUI `ThemeBrush`) |
| 테마 토큰 | `Palette::dark()/light()` | `palette(&theme)` — `bg: surface` → `theme.popover` | 없음 — `css!`/`class`를 읽지 않는다 |
| 상태 소유 | 앱이 `Ctx`를 소유 | `ElmView` 엔티티가 소유 | `ElmView` 컴포넌트가 소유(`RefCell<Ctx>`) |
| `<Raw>` payload | `&mut egui::Ui` | `&mut gpui_kit::Window` | `&mut Option<windows_reactor::View>` (**뷰를 돌려준다**) |
| 헤드리스 테스트 | `mount!` + `egui::Context::run_ui` | `#[gpui_kit::test]` + `TestAppContext` | `mount!` + `plan()` (플랫폼 독립 계획 층) |
| 검증용 관찰 | `Pass { buttons, checks, styles }` | `Pass { styles, labeled, inputs }` | `Pass { controls, labeled, texts, inputs, dialogs }` |
| 효과/스트림 구동 | 앱이 직접 (`take_due`) | 앱이 직접 (`take_due`) | 어댑터의 `drive` 모듈 (+`update`에서 자동) |

## 읽는 순서 (권장)

- **처음이라면**: `egui/01` → `egui/04` → `egui/07` → `gpui/01` → `gpui/02`
- **상태/비동기가 궁금하면**: `egui/08`, `egui/09`, `egui/10`, `egui/11`, `egui/12`
- **성능이 궁금하면**: `egui/17`, `gpui/16`, 그리고 `crates/elm-magic-egui/benchmark/README.md`
- **스타일을 `css!` 없이 쓰려면**: `gpui/16` (스타일 = `style::register`, `css!`는 그 호출을
  시작 섹션에서 대신 해 주는 매크로일 뿐)
- **스타일 계층 없이 WinUI로 쓰려면**: `windows-reactor/01` → `08`(Raw) → `16`(스타일) → `20`
- **플랫폼 호스트와 elm을 잇는 법**: `windows-reactor/11`(props/콜백 prop), `gpui/12`(구독)
- **앱 골격이 필요하면**: `egui/20`, `gpui/20`, `windows-reactor/20`

## 버전

- `elm-magic` / `elm-magic-egui` / `elm-magic-gpui` / `elm-magic-windows-reactor` = 0.8.5 (이 저장소)
- `egui` 0.36.2 (`elm-magic-egui`의 의존성), eframe은 같은 마이너(0.36)
- `gpui-kit` 0.6 (0.6.2~0.6.4 실측)
- `windows-reactor` 0.100.0 (0.100.0 소스 실측 — WinUI 3 / Windows App SDK, **Windows 전용**)
