#!/usr/bin/env sh
# Iris PR에서 쓰는 기본 GitHub 라벨 세트를 생성하거나 갱신한다.
set -eu

repo="${1:-ohah/iris}"

ensure_label() {
  name="$1"
  color="$2"
  description="$3"

  if gh label list --repo "$repo" --limit 200 --json name --jq '.[].name' | grep -Fxq "$name"; then
    gh label edit "$name" --repo "$repo" --color "$color" --description "$description" >/dev/null
    echo "update $name"
  else
    gh label create "$name" --repo "$repo" --color "$color" --description "$description" >/dev/null
    echo "create $name"
  fi
}

ensure_label "ci" "ededed" "CI, GitHub Actions, branch protection"
ensure_label "docs" "ededed" "Documentation and agent guidance"
ensure_label "tests" "1d76db" "Tests and verification coverage"
ensure_label "tooling" "5319e7" "Developer tooling, package manager, formatter, linter"
ensure_label "runtime" "0e8a16" "Iris runtime architecture and execution path"
ensure_label "jsi" "1D76DB" "React Native JSI and native boundary"
ensure_label "react-native" "0e8a16" "React Native app, Fabric, TurboModules, Metro"
ensure_label "hermes" "5319e7" "Hermes compatibility and baseline behavior"
ensure_label "quickjs" "fbca04" "QuickJS backend experiment"
ensure_label "performance" "d93f0b" "Performance budgets and regression prevention"
ensure_label "benchmark" "d93f0b" "Benchmark apps and measurement scenarios"
ensure_label "rust" "dea584" "Rust crates, cxx, memory ownership"
ensure_label "dependencies" "0366d6" "Dependency and lockfile changes"
