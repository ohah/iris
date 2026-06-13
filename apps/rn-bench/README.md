# Iris React Native Benchmark App

This app is the React Native/Hermes baseline for Iris runtime experiments.

## Stack

- React Native 0.85
- React 19.2
- Hermes enabled
- New Architecture enabled
- Bun-managed workspace

## Commands

Run from the repository root.

```sh
mise run install
mise run rn-start
mise run rn-ios
mise run rn-android
mise run rn-typecheck
mise run rn-test
```

iOS needs CocoaPods before the first native build.

```sh
cd apps/rn-bench/ios
bundle install
bundle exec pod install
```

## Scope

This app should stay compatible with the official React Native/Hermes path. Iris-specific runtime experiments should be added as isolated comparison paths instead of replacing the baseline.
