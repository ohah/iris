# 파일/폴더 구조

```text
.
├── apps/
│   └── rn-bench/   # React Native 0.85 Hermes 기준선 앱
├── crates/
│   ├── iris-core/  # Iris 공통 계약과 metadata
│   ├── iris-jsi/   # React Native JSI 경계
│   └── iris-qjs/   # QuickJS backend 실험 경계
├── docs/           # 개발 규칙과 PR 기준
├── references/     # gitignored 외부 런타임 레퍼런스
├── tools/          # 레퍼런스, 라벨, 성능 게이트 보조 스크립트
└── packages/       # 향후 JS/TS 패키지 workspace
```

## 경계

- `iris-core`는 플랫폼과 엔진 backend에 의존하지 않는다.
- `iris-jsi`는 C++ JSI와 Rust-owned 상태를 잇는 경계다.
- `iris-qjs`는 QuickJS backend 실험을 캡슐화한다. production 경로로 승격하려면 별도 호환성 근거가 필요하다.
- React Native PoC 앱은 `apps/rn-bench`에 둔다. 이 앱은 Hermes 순정 기준선이며 Iris 경로는 이후 같은 앱에 별도 비교 경로로 붙인다.
- 외부 런타임 소스는 `references/` 아래에만 둔다. 이 디렉터리는 gitignore된다.
- `tools/perf/`는 branch protection에 연결된 성능 예산 게이트를 둔다.
