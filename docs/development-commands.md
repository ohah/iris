# 개발 명령

Iris는 `mise`를 개발 명령의 단일 진입점으로 둔다.

## 준비

```sh
mise install
mise run install
mise run fetch-references
mise run sync-labels
```

## 자주 쓰는 명령

```sh
mise run fmt
mise run fmt-check
mise run lint
mise run vp-check
mise run fetch-references
mise run sync-labels
mise run perf
mise run rust-check
mise run test
mise run check
```

## 원칙

- JavaScript와 TypeScript는 Vite+와 VoidZero/Oxc 계열인 `oxlint`와 `oxfmt`를 사용한다.
- Rust는 별도 `rustfmt.toml` 없이 기본 `rustfmt` 스타일을 사용한다.
- Bun은 `package.json`의 `packageManager`와 `.mise.toml`의 `bun = "1.3.14"`로 고정한다.
- Vite+는 로컬 `vite-plus` 패키지와 `bunx vp`로 실행한다.
- `mise run check`가 PR 전 기본 검증 경로다.
