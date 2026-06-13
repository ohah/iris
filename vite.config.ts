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
      "rn-android": "bun run --cwd apps/rn-bench android",
      "rn-ios": "bun run --cwd apps/rn-bench ios",
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
