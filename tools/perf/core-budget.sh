#!/usr/bin/env sh
# Iris runtime benchmark가 들어오기 전까지 branch protection의
# `core performance budget` 체크 이름과 산출물 계약을 고정한다.
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/artifacts/perf"
mkdir -p "$out"

cat >"$out/core-budget.json" <<'JSON'
{
  "status": "placeholder",
  "project": "iris",
  "check": "core performance budget",
  "runtime_benchmarks": false,
  "message": "React Native benchmark harness exists, but Iris JSI runtime budgets are not implemented yet."
}
JSON

printf '%s\n' "core performance budget: placeholder gate passed"
