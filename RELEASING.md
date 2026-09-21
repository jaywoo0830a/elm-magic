# Releasing

```
bash ./release 1.2.3
```

## 크레이트 (배포 순서)

| 순서 | 크레이트 | 비고 |
|---|---|---|
| 1 | `elm-magic-macros` | 색인 전파를 기다린다 |
| 2 | `elm-magic` | 색인 전파를 기다린다 |
| 3 | `elm-magic-egui` | 어댑터 셋은 서로 독립 |
| 4 | `elm-magic-gpui` | |
| 5 | `elm-magic-windows-reactor` | WinUI 3 — 의존성은 `[target.'cfg(windows)'.dependencies]` 라서 **리눅스/macOS 에서도 포장·배포가 통과**한다 |

`bash ./release <버전> --dry-run` 으로 범프 대상·포장·배포 계획을 먼저 확인할 수 있다
(`--local` 이면 푸시/배포 없이 커밋·태그까지만).
