import { defineConfig } from "vite-plus";

export default defineConfig({
  fmt: {
    ignorePatterns: ["node_modules/**", "target/**", "dist/**", "build/**", "coverage/**"],
    printWidth: 100,
    trailingComma: "all",
  },
  lint: {
    ignorePatterns: ["node_modules/**", "target/**", "dist/**", "build/**", "coverage/**"],
  },
  run: {
    tasks: {
      check: {
        command: [
          "vp check",
          "cargo fmt --all --check",
          "cargo check --workspace --all-targets",
          "cargo test --workspace",
          "bun run --cwd apps/rn-bench typecheck",
          "bun run --cwd apps/rn-bench test --runInBand",
        ],
      },
      "bench-extract-fixture": "bun run bench:extract-hermes:fixture",
      "bench-extract-hermes":
        "bun run bench:extract-hermes --input=artifacts/bench/metro-hermes.log --output=artifacts/bench/hermes-baseline.json",
      "bench-extract-hermes-release":
        "bun run bench:extract-hermes:release --input=artifacts/bench/rn-release-hermes.log --output=artifacts/bench/hermes-release-baseline.json",
      "bench-extract-release-fixture": "bun run bench:extract-hermes:release-fixture",
      "bench-js": "bun run bench:js",
      "bench-smoke": "bun run bench:smoke",
      "rn-codegen":
        "rm -rf artifacts/codegen/rn-bench && bun run --cwd apps/rn-bench react-native codegen --platform all --outputPath ../../artifacts/codegen/rn-bench",
      "rn-android": "bun run --cwd apps/rn-bench android",
      "rn-android-build-debug":
        "cd apps/rn-bench/android && ./gradlew :app:assembleDebug -x lint -x test",
      "rn-android-build-release":
        "cd apps/rn-bench/android && ./gradlew :app:assembleRelease -x lint -x test",
      "rn-ios": "bun run --cwd apps/rn-bench ios",
      "rn-ios-build-debug":
        "cd apps/rn-bench/ios && RCT_NO_LAUNCH_PACKAGER=1 xcodebuild -workspace IrisBench.xcworkspace -scheme IrisBench -configuration Debug -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build",
      "rn-ios-build-release":
        "cd apps/rn-bench/ios && RCT_NO_LAUNCH_PACKAGER=1 xcodebuild -workspace IrisBench.xcworkspace -scheme IrisBench -configuration Release -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build",
      "rn-ios-pods": "cd apps/rn-bench/ios && bundle install && bundle exec pod install",
      "rn-start": "bun run --cwd apps/rn-bench start",
      "rn-test": "bun run --cwd apps/rn-bench test --runInBand",
      "rn-typecheck": "bun run --cwd apps/rn-bench typecheck",
      "rust-check": "cargo check --workspace --all-targets",
      "rust-test": "cargo test --workspace",
      "rust-fmt-check": "cargo fmt --all --check",
      "rust-fmt": "cargo fmt --all",
    },
  },
});
