# 참고 예제 (컴파일 대상 아님)

`elm-magic`을 **egui**(`elm-magic-egui`), **gpui-kit**(`elm-magic-gpui`),
**WinUI 3 / windows-reactor**(`elm-magic-windows-reactor`)에서 각각 어떻게 쓰는지
정리한 베스트 패턴 모음이다. 예제 20개씩, 총 60개.

```
examples/
├── README.md               ← 지금 이 파일
├── egui/01_…20_*.rs        ← egui 어댑터 + eframe/egui 앱 패턴
├── gpui/01_…20_*.rs        ← gpui-kit 어댑터 + ElmView 패턴
└── windows-reactor/        ← WinUI 3 어댑터 (스타일 계층 없음)
    ├── 01_…20_*.rs         ←   예제 20개 (`fn main()` + WinUI 호스트)
    ├── Cargo.toml          ←   예제 전용 패키지 — `[[example]]` 20개 등록 + `winui` 게이트
    └── run-tests.ps1       ←   Windows 실행/테스트 편의 스크립트
```

**windows-reactor 세트는 실행 가능하다** — 자기 매니페스트(`windows-reactor/Cargo.toml`)에
등록되어 있어 Windows에서 테스트·실행된다(아래
"## Windows에서 windows-reactor 예제 돌리기"). egui/gpui 세트는 지금처럼 문서 전용이다.

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

**예외 하나 — `windows-reactor/`**: 이 세트만 `examples/windows-reactor/Cargo.toml`
(독립 패키지)에 `[[example]]`로 **명시 등록**되어 있어 Windows에서 실제로
컴파일·테스트·실행된다. 타겟은 `winui` 기능으로 게이트했기 때문에, 그 기능이 꺼진
리눅스/macOS에서는 **전부 건너뛰어진다**(warning + exit 0) — 즉 CI와 루트
`cargo test --workspace`는 이전과 똑같이 이 예제들을 건드리지 않는다.
→ "## Windows에서 windows-reactor 예제 돌리기" 참고.

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

## Windows에서 windows-reactor 예제 돌리기

windows-reactor 세트는 **문서가 아니라 실제로 도는 코드**다 — 각 파일에
`#[cfg(test)] mod tests`가 있고, `examples/windows-reactor/Cargo.toml`이 그 파일들을
Cargo 타겟으로 등록한다. 저장소 루트에서:

```powershell
# 20개 전부 — 각 예제를 테스트 하네스로 컴파일해 안의 #[cfg(test)]를 실행한다
cargo test --examples --features winui --manifest-path examples\windows-reactor\Cargo.toml

# 하나만
cargo test --example 09_async_effects --features winui --manifest-path examples\windows-reactor\Cargo.toml

# 실제 창 띄우기 (WinUI 3 / WASDK 런타임 필요)
cargo run --example 09_async_effects --features winui --manifest-path examples\windows-reactor\Cargo.toml
```

같은 일을 하는 편의 스크립트도 있다(`examples/windows-reactor/run-tests.ps1`):

```powershell
cd examples\windows-reactor                     # 이 디렉터리에서 바로 돌려도 된다
powershell -ExecutionPolicy Bypass -File run-tests.ps1                    # 20개 테스트
powershell -ExecutionPolicy Bypass -File run-tests.ps1 -List              # 목록+제목, 매니페스트 누락 경고
powershell -ExecutionPolicy Bypass -File run-tests.ps1 -Example 09        # 하나만
powershell -ExecutionPolicy Bypass -File run-tests.ps1 -Example 20 -Run   # 창
powershell -ExecutionPolicy Bypass -File run-tests.ps1 -Check             # 컴파일만
powershell -ExecutionPolicy Bypass -File run-tests.ps1 -Filter counter    # 테스트 이름 필터
```

**예제 하나만 창으로 띄우려면** `run-example.ps1`이 더 짧다 — 고른 예제만 빌드해
실행하고, 창을 닫으면 끝난다(테스트 타겟은 거치지 않는다):

```powershell
.\run-example.ps1 18            # 번호(또는 이름 접두어) 하나만
.\run-example.ps1 -Example 09
.\run-example.ps1 20 -Release   # 릴리스 빌드
.\run-example.ps1               # 인자 없이: 목록을 보여주고 번호를 물어본다
```

- **`--features winui`가 필요한 이유**: 예제 타겟이 그 기능으로 게이트되어 있다.
  기능을 **끄면** 리눅스/macOS에서 다음이 조용히 끝난다(exit 0):
  `cargo test --examples --manifest-path examples\windows-reactor\Cargo.toml` →
  `target filter ... no targets matched` — 그래서 CI와 루트 `cargo test --workspace`에는
  영향이 없다. 반대로 기능을 **켜면** 리눅스에서는 컴파일이 실패한다
  (그 플랫폼에는 `windows-reactor` 의존성이 없다).
- `run-tests.ps1`은 Windows가 아니면 경고만 하고 `exit 2` 한다. 하는 일은 cargo
  호출뿐이다(파일을 만들거나 지우지 않는다).
- **새 예제를 추가하면** `examples/windows-reactor/Cargo.toml`의 `[[example]]`도 함께
  추가한다(`run-tests.ps1 -List`가 누락/유령 항목을 경고한다).
- 호스트(WinUI) 구역을 들어내면 `#[cfg(test)] mod tests`는 **플랫폼 독립 코어**
  (`mount!`/`flush`/`advance`/`plan()`)만 쓰므로 리눅스에서도 그대로 돈다 —
  그래서 이 스위트는 "Windows가 없으면 검증 불가"가 아니라 "Windows가 없으면
  창만 못 띄운다"에 가깝다.

## 예제를 실제로 돌리려면

컴파일 대상이 아니므로 "실행"하려면 앱 쪽에서 다음을 갖춰야 한다.

| 예제 | 추가로 필요한 것 |
|---|---|
| `egui/*` | `eframe = "0.36"`(egui 0.36과 같은 마이너), `elm-magic-egui` |
| `gpui/*` | `gpui-kit = "0.6"`(+ 테스트는 dev-dependencies에서 `features = ["test-support"]`) |
| `windows-reactor/*` | `windows-reactor = "0.100"` + `elm-magic-windows-reactor` (**Windows 전용**, WASDK 런타임 필요) — 이 저장소에는 이미 준비된 매니페스트(`windows-reactor/Cargo.toml`)와 스크립트가 있다 |
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

- `elm-magic` / `elm-magic-egui` / `elm-magic-gpui` / `elm-magic-windows-reactor` = 0.8.6 (이 저장소)
- `egui` 0.36.2 (`elm-magic-egui`의 의존성), eframe은 같은 마이너(0.36)
- `gpui-kit` 0.6 (0.6.2~0.6.4 실측)
- `windows-reactor` 0.100.0 (0.100.0 소스 실측 — WinUI 3 / Windows App SDK, **Windows 전용**)
