# Iris

Iris is a React Native JavaScript runtime experiment focused on preserving
React Native 0.85 and Hermes V1 observable compatibility before making
performance claims.

## Tooling

- `mise` pins Bun, Node.js, and Rust.
- Bun is the project package manager through `packageManager`.
- Vite+ is available through the local `vite-plus` package and `bunx vp`.
- `oxlint` and `oxfmt` provide VoidZero/Oxc JavaScript and TypeScript linting
  and formatting.
- Rust uses the default `rustfmt` style.

```sh
mise install
mise run install
mise run check
```
