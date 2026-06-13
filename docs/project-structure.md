# 파일/폴더 구조

```text
.
├── crates/
│   ├── iris-core/  # Iris 공통 계약과 metadata
│   ├── iris-jsi/   # React Native JSI 경계
│   └── iris-qjs/   # QuickJS backend 실험 경계
├── docs/           # 개발 규칙과 PR 기준
├── references/     # gitignored 외부 런타임 레퍼런스
├── packages/       # 향후 JS/TS 패키지 workspace
└── apps/           # 향후 React Native PoC 앱
```

## 경계

- `iris-core`는 플랫폼과 엔진 backend에 의존하지 않는다.
- `iris-jsi`는 C++ JSI와 Rust-owned 상태를 잇는 경계다.
- `iris-qjs`는 QuickJS backend 실험을 캡슐화한다. production 경로로 승격하려면 별도 호환성 근거가 필요하다.
- React Native PoC 앱은 `apps/` 아래에 둔다.
- 외부 런타임 소스는 `references/` 아래에만 둔다. 이 디렉터리는 gitignore된다.
