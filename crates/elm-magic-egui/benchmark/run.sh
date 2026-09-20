#!/usr/bin/env bash
#
# 이 디렉터리에서 바로 돌리는 얇은 껍데기 — 루트의 `bench`를 그대로 부른다.
#
#   cd crates/elm-magic-egui/benchmark && bash run.sh 1000 --nocapture
#
# 루트에서 부르는 쪽이 기본이다: `bash ./bench`.
exec bash "$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)/bench" "$@"
