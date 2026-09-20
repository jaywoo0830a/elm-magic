# CHANGELOG

## [0.8.3] — 2026-09-20

### Changed

- 0.8.3

## [0.8.2] — 2026-09-20

### Changed

- patch

## [0.8.1] — 2026-09-19

### Changed

- 0.8.1

## [0.8.0] — 2026-09-19

### Changed

- fix bugs
- done
- add faetures
- 0.8.0 progress 1
- add preview
- commit

## [Unreleased] — 0.8.x 조건/반복 + `IntoView`

### Performance — egui 어댑터

측정 중심의 벤치마크(`crates/elm-magic-egui/benchmark/`, 한 줄 실행 `bash ./bench`)로
병목을 잡아 다음을 고쳤다. 참조 화면(1,000행 목록 = 3,005노드)에서 어댑터 한 프레임
**4.02ms → 2.78ms (-31%)**, 3,000행(9,005노드)에서 **≈13.5 → 8.36ms (-38%)**.
손으로 쓴 egui 대비 **3.6배 → 2.5배**이고, 3,000행에서 60fps 여유가 74 → 119fps가 됐다.
남은 비용의 정체는 "노드마다 만드는 egui `Ui` 조작 수"로 귀속된다(그 문서 §6).

- `elm_magic_egui::render_fast` 추가 — `Pass`의 기록(`styles`/`buttons`/`checks`)을
  채우지 않는 **앱 경로**. 그 기록은 노드마다 `ResolvedStyle`(약 600바이트)과 버튼
  라벨 `String`을 복제해 프레임당 약 2MB를 memcpy했다(3,005노드에서 복제 3,004회;
  9,005노드에서는 -0.9ms). 스타일/버튼 전달을 검증하는 테스트·디버깅은 기존
  `render`/`render_with_palette`를 그대로 쓴다(계약 유지).
- **프레임 + 레이아웃을 한 스코프로** — `Frame::show` + `ui.with_layout`이 컨테이너마다
  자식 `Ui`를 두 번 만들던 것을, egui 0.36의 공개 부품(`Frame::total_margin`/
  `widget_rect`/`paint` + `UiBuilder::layout`)으로 같은 그림을 **한 스코프**에 그린다
  (행마다 -240ns). `apply_size`는 부모 주축을 인자로 받는다.
- **상태 추적 게이트** (`style::needs_pointer_state`) — `:hover`/`:active`/`:focus`를
  요구하는 규칙이 하나도 없으면 노드마다 하던 egui temp data 읽기/쓰기 + 컨텍스트 락
  3~4회를 건너뛴다(`:disabled`는 요소 자체에서 오므로 무관). 포인터 상태도 패스당
  한 번만 읽는다. 규칙이 있는 앱은 이전과 똑같이 동작하며, `benchmark_state`가
  양방향(없으면 안 읽고, 있으면 읽는다)을 고정한다.
- **스타일 해석 캐시** — 한 프레임에 같은 노드를 2~3회 해석하던 반복(intrinsic 예산 →
  자식 루프 → 그리기)을 1회로. 해석은 호출마다 전역 등록부를 훑어 **등록 규칙당 약
  5ns**가 들므로(실측) 규칙이 200개인 앱에서 약 6ms, 1,000개면 약 40ms를 없앤다.
  캐시 `HashMap`은 스레드 지역에 두고 패스 동안 소유권만 옮겨 쓴다 — 매 프레임
  새로 잡으면 9,000노드에서 5MB대 새 페이지(페이지 폴트)가 프레임 비용이 된다.
- **주축 예산 프리패스 단락** — `justify`/`wrap`/내용 폭 고정/`align-self`가 없고
  가변 자식(`fill`/`flex-grow`)도 없으면 `child_budgets`(자식 트리 재순회 + 텍스트
  측정)를 건너뛴다. 결과가 쓰이지 않는 계산이었다.
- **`Frame::NONE` → `ui.scope`** — 배경/여백/모서리/테두리/그림자가 하나도 선언되지
  않은 컨테이너는 `Frame`을 만들지 않는다 (`Frame::NONE.show` 425ns vs `ui.scope` 273ns).

### Added

- `IntoView`(§3.2) — children 자리의 `{expr}`이 `IntoView`를 구현한 값이면 무엇이든 그린다.
  `&str`/`String`/숫자/`bool`은 텍스트, `Element`는 그대로, `Option<T>`는 `None`이면
  미렌더, `Vec<T>`/`Iterator<Item = T>`는 펼쳐서 그린다. 기존 `{if ..}` / `{items.map(..)}`도
  그대로 동작한다(autoref 특수화로 `IntoIterator`와 스칼라를 함께 지원).
  `IntoView`에는 `#[diagnostic::on_unimplemented]` DSL 메시지를 달았다.
- `<If when={..}>` / `<Else>` — 조건 분기. `<Else>` 생략 시 거짓이면 아무것도 그리지 않는다.
  중첩/동적 상태 변화/본문 전체 분기를 지원한다.
- `<For each={..} as={..} key={..}>` — 반복. `each`는 `IntoIterator`, `as`는 아이템 바인딩.
  `key`가 없으면 위치 기반, 있으면 keyed 슬롯(자식 상태가 키를 따라간다).
- `<Switch on={..}>` / `<Case when={..}>` / `<Default>` — 패턴 분기.
  `when`은 패턴이라 `Status::Failed(e)` 바인딩을 쓸 수 있다. `<Default>` 생략 시
  일치하는 분기가 없으면 아무것도 그리지 않는다.
- `<> … </>` — 레이아웃 없는 프래그먼트 (본문/조건/반복 안쪽 어디서나).
- 제어 흐름 태그는 **레이아웃 노드를 만들지 않는다** — 감싼 자식이 그대로 트리에 나온다.
  기존 `{if …}` / `{items.map(…)}` 문법과 공존한다 (추가 중심).
- `css!` 확장 속성 26종 (총 62종) — `flex-direction` `flex-grow` `flex-shrink`
  `align-self` `overflow` `aspect-ratio` `z-index`, 개별 `padding-*`/`margin-*`,
  `border-style`/`border-*-width`, `white-space` `text-overflow` `max-lines`,
  `rotate` `scale` `pointer-events`. `padding: 8; padding-left: 4`처럼 숏핸드와
  개별 값이 함께 산다. `crates/elm-magic-macros` 쪽도 대응하는 `Kind`/검증을 갖는다.
- egui 어댑터가 확장 속성을 그린다 — 위 목록 + (`border-style: dashed/dotted`는
  실선, 개별 `border-*-width`는 최댓값, `max-lines`는 잘라내기, `scale`은 좌상단
  기준 시각 변환, `z-index > 0`은 전경 레이어로 근사. `rotate`는 egui
  `TSTransform`에 회전이 없어 미반영).

### Fixed

- egui 어댑터 레이아웃 버그 4건 (`crates/elm-magic-egui/tests/elm_bugs_new.rs`,
  `elm_wrap_tests.rs`가 회귀 센티넬):
  - **1b/1c** `wrap: true` 행의 자식이 중첩 컨테이너일 때 줄바꿈이 무시되던 문제 —
    egui의 줄바꿈 판단은 그리기 전에 알려진 크기로만 이뤄지므로, 컨테이너 자식의
    intrinsic 크기를 미리 예약(`allocate_exact_size` + `scope_builder`)한다.
  - **2** 내용 크기 세로 컨테이너가 가로 부모(행) 안에 있을 때 자식의 `width: fill`이
    남은 창 폭을 먹어 조상이 팽창하던 문제 — 교차축(폭)을 내용 폭으로 고정한다.
  - **4** `flex-grow: 1`이 텍스트 노드에서 무시되던 문제 — 예산만큼 자리를 예약해
    그린다(egui는 "실제 사용한" 크기만 차지하므로 최소 크기를 못박는다).
  - **6** 가로 행의 `align-self: center`가 남은 세로를 전부 먹어 다음 행이 밀리던
    문제 — 행 높이(자식들의 최대 높이) 안에서 정렬한다.
- `crates/elm-magic-egui/tests/elm_bugs_new.rs`·`elm_wrap_tests.rs`가 `eframe`을
  import해 컴파일이 안 되던 문제(이 크레이트의 의존성은 `egui`뿐). `egui::` 경로만 쓴다.
- wrap/fill 재현 테스트의 이름과 헤더 표가 실제 관측과 어긋나던 것.

- `view!` 파라미터 타입 직렬화가 토큰별 `to_string()`+공백 결합이라
  `Vec<elm_magic::Element>`가 `Vec < elm_magic : : Element>`로,
  `&'static str`이 `& ' static str`로 깨지던 문제. `render_tokens`로 통일했다.

### Tests

- `crates/elm-magic-egui/benchmark/` — **측정 중심 벤치마크** (계약 스위트와 분리;
  `Cargo.toml`의 `[[test]]`로 명시 등록, `benchmark/README.md`에 방법론·실측·병목 귀속).
  단계 분해(`benchmark_phases`: P0~P5), 손 egui 기준선(`benchmark_baselines`: naive/tuned
  vs 어댑터 + 페인트 명령 수), 핫스팟(`benchmark_hotspots`: H1~H8 — 해석 단가, 깊이,
  keyed 슬롯, 클래스 매칭, egui temp 왕복, `Pass` 복제, **egui 조작 단가표**),
  규칙 수 스윕(`benchmark_rules`), 구조 회귀 가드(`benchmark_guard`: 트리 모양 3n+5,
  기록 ≤ 노드 수, 프레임 반복 시 egui temp data 불변 + `ELM_MAGIC_PERF=1`일 때만 시간 예산).

- `tests/0.8/` 계약 테스트 6종(44개): `v0_8_if_else`, `v0_8_for_loop`, `v0_8_switch`,
  `v0_8_fragment`, `v0_8_into_view`, `v0_8_interop`. 구현 현황은 `tests/0.8/README.md` 참고.

## [0.7.6] — 2026-09-19

### Changed

- fix bug
- ss

## [0.7.5] — 2026-09-19

### Changed

- add ss
- commit
- add some tests

