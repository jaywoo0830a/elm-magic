# CHANGELOG

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

