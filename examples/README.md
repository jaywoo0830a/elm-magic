# 참고 예제 (컴파일 대상 아님)

`elm-magic`을 **egui**(`elm-magic-egui`)와 **gpui-kit**(`elm-magic-gpui`)에서 각각
어떻게 쓰는지 정리한 베스트 패턴 모음이다. 예제 20개씩, 총 40개.

```
examples/
├── README.md            ← 지금 이 파일
├── egui/01_…20_*.rs     ← egui 어댑터 + eframe/egui 앱 패턴
└── gpui/01_…20_*.rs     ← gpui-kit 어댑터 + ElmView 패턴
```

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
| 공통 | `elm-magic` (워크스페이스 경로 의존이면 `path = "…"`) |

그리고 **앱 루프를 직접 만들어야 한다** — 아래 두 가지는 어댑터가 해 주지 않는다.

## 반드시 알아야 할 두 가지 (예제에서 가장 자주 걸리는 함정)

1. **`<-`(효과)와 `->`(스트림)는 앱이 구동해야 한다.**
   `frame`/`render`는 트리만 그린다. 효과는 `Arena::take_due()`, 스트림은
   `Arena::take_streams()`/`push_streams()`, 시계는 `Arena::set_now(ms)`가 필요하다.
   테스트의 `flush`/`advance`/`pump`가 하는 일이 정확히 이것이다.
   구현은 `egui/09`·`egui/10`·`gpui/11`·`gpui/13`·`gpui/20`에 있다.
2. **`on_key`는 앱이 키를 디스패치해야 한다.**
   핸들러는 `Ctx::keys`(`Vec<(String, KeyHandler)>`)에 쌓인다 —
   테스트의 `press_key("Ctrl+S")`처럼 앱이 찾아 호출한다(`gpui/13`의 `dispatch_key`).
   `on_tick`도 같은 원리로 시계가 있어야 돈다.

## 두 어댑터의 공통 계약

둘 다 **코어가 `ResolvedStyle`까지 해석**하고, 어댑터는 그 결과를 옮기기만 한다.
그래서 예제의 컴포넌트(`view!` + `css!`)는 **어느 쪽에서도 그대로 재사용**된다.

| | egui | gpui-kit |
|---|---|---|
| 진입점 | `elm_magic_egui::render(ui, &tree, &mut ctx.arena)` | `ElmView::<C>::new(cx)` (gpui `Render` 구현) |
| 앱 경로(빠름) | `render_fast` (기록 안 함) | `Pass`는 항상 채우지만 노드당 1회 push |
| 팔레트 출처 | `Visuals::dark_mode` 또는 인자 | `ActiveTheme`(gpui-kit 테마) |
| 테마 토큰 | `Palette::dark()/light()` | `palette(&theme)` — `bg: surface` → `theme.popover` |
| 상태 소유 | 앱이 `Ctx`를 소유 | `ElmView` 엔티티가 소유 |
| `<Raw>` payload | `&mut egui::Ui` | `&mut gpui_kit::Window` |
| 헤드리스 테스트 | `mount!` + `egui::Context::run_ui` | `#[gpui_kit::test]` + `TestAppContext` |
| 검증용 관찰 | `Pass { buttons, checks, styles }` | `Pass { styles, labeled, inputs }` |

## 읽는 순서 (권장)

- **처음이라면**: `egui/01` → `egui/04` → `egui/07` → `gpui/01` → `gpui/02`
- **상태/비동기가 궁금하면**: `egui/08`, `egui/09`, `egui/10`, `egui/11`, `egui/12`
- **성능이 궁금하면**: `egui/17`, `gpui/16`, 그리고 `crates/elm-magic-egui/benchmark/README.md`
- **스타일을 `css!` 없이 쓰려면**: `gpui/16` (스타일 = `style::register`, `css!`는 그 호출을
  시작 섹션에서 대신 해 주는 매크로일 뿐)
- **앱 골격이 필요하면**: `egui/20`, `gpui/20`

## 버전

- `elm-magic` / `elm-magic-egui` / `elm-magic-gpui` = 0.8.5 (이 저장소)
- `egui` 0.36.2 (`elm-magic-egui`의 의존성), eframe은 같은 마이너(0.36)
- `gpui-kit` 0.6 (0.6.2~0.6.4 실측)
