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
- React Native PoC 앱은 `apps/rn-bench`에 둔다. 이 앱은 Hermes 순정 기준선이며 초기 Iris native module probe는 같은 앱의 별도 benchmark case로 붙인다. 엔진 대체 비교는 독립 앱 복제가 아니라 같은 앱 소스의 Android `engine` flavor별 release variant로 분리한다. Android Iris 엔진 AAR skeleton은 `apps/rn-bench/android/iris-engine`에 두고, 계약은 `docs/iris-android-engine-contract.md`에 둔다.
- 외부 런타임 소스는 `references/` 아래에만 둔다. 이 디렉터리는 gitignore된다.
- `tools/perf/`는 branch protection에 연결된 성능 예산 게이트를 둔다.
