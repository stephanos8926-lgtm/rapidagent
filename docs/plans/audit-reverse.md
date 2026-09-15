# Reverse Audit: RapidAgent Framework Architecture Component List

## Audit Scope
- **Document**: `docs/plans/architecture-component-list.md` (v1.0)
- **Auditor**: Subagent (Reverse View)
- **Date**: 2026-09-15

## Methodology
Reverse audit examines: Are there unnecessary components? Are dependencies correct? Is the architecture over-engineered? What risks does the current design introduce?

---

## Findings

### ❌ UNNECESSARY COMPONENTS

| Component | Reason for Removal | Alternative |
|-----------|-------------------|-------------|
| `FWD-006` `rapidagent-cli` | Early CLI may duplicate `ragc` (DX-001). Can be folded into daemon or separate repo. | Merge into `DRN-001` or defer to Horizon 2. |
| `OBS-004` Dashboard | Grafana dashboards are operational, not architectural. Can be created post-deployment. | Remove; create as separate repo under `rapidagent-dashboards`. |
| `MEM-002` In-Process Deriver | Honcho fork already has this. Duplicate effort. | Remove; use Honcho directly. |
| `MEM-004` Rate Limiter | Token-bucket limiter is minor utility. Can be part of `MEM-001` Honcho Integration. | Fold into Honcho Integration. |

### ⚠️ OVER-ENGINEERING RISKS

| Component | Risk | Mitigation |
|-----------|------|------------|
| `CMP-006` Binary Renderer | Adds complexity early; primary output is folder-based. | Keep as P2 but defer implementation to Horizon 3. |
| `SEC-004` Compliance Report | SOC2/ISO27001 is Phase 4. Too early to design. | Remove from component list; track in roadmap only. |
| `DX-002` IDE Extension | TypeScript VS Code extension is a separate product. | Move to Horizon 3; not needed for core framework. |
| `DRN-005` Config Watcher | File watcher adds complexity; can use SIGHUP + restart. | Defer to P2; implement if operator feedback demands hot-reload. |

### ⚠️ DEPENDENCY ANTI-PATTERNS

| Issue | Components Involved | Problem | Recommendation |
|-------|---------------------|---------|----------------|
| Circular Dependency Risk | `ENG-004` Policy Hook ↔ `WRD-001` Security Warden | Engine calls Warden; Warden may need engine context (e.g., session ID). | Define clear interface: Warden receives `CheckRequest` (not engine ref). Use message-passing, not direct calls. |
| Tight Coupling | `POL-004` POL Subscriber ↔ `ENG-003` Event Bus Client | Both subscribe to same NATS subjects. Risk of double-processing. | Define ownership: Engine owns `agents.>.events`; POL owns `pol.>.interventions`. Separate subjects. |
| Singleton Overuse | `POL-001`, `WRD-001`, `DRN-001` all use singleton pattern | Hard to test, hard to parallelize, hard to replace. | Use dependency injection; make singletons explicit via `Context` object. |

### ⚠️ MISSING FAILURE MODES

| Failure Mode | Affected Components | Mitigation Gap |
|--------------|---------------------|----------------|
| NATS cluster loss | `FWD-003`, `ENG-003`, `POL-004`, `WRD-001` | No explicit "offline mode" component. Add `GAP-004` (Fallback Executor) from forward audit. |
| PostgreSQL failure | `POL-003`, `MEM-001`, `SEC-002` | No local fallback storage. Add `POL-008`: Local Intervention Store (JSON file, sync to PG when available). |
| Warden timeout | `WRD-005` | Already noted (fail-closed). Good. |
| POL Agent hallucination | `POL-006` (Phase 2) | Not yet designed. Add `POL-009`: POL Agent Safety Gate (rules-first, LLM-advisory-only). |

### ⚠️ SECURITY CONCERNS

| Concern | Components | Recommendation |
|---------|------------|----------------|
| Ed25519 key storage | `CMP-005`, `SEC-001` | Keys must never leave HSM/Vault. Add `SEC-005`: Key Storage Policy. |
| NATS auth | `INF-001`, `FWD-003` | Enable ACLs, per-subject permissions. Add `SEC-006`: NATS Security Configuration. |
| Agent sandboxing | `ENG-001`, `WRD-001` | File/network isolation for untrusted agents. Add `SEC-007`: Seccomp/AppArmor Profiles. |

---

## Summary Statistics

| Category | Count |
|----------|-------|
| Unnecessary Components | 4 |
| Over-Engineering Risks | 4 |
| Dependency Anti-Patterns | 3 |
| Missing Failure Modes | 4 |
| Security Concerns | 3 |

---

## Recommendations

1. **Remove 4 components**: CLI (merge), Dashboard (defer), Deriver (use Honcho), Rate Limiter (fold into Honcho)
2. **Defer 4 components**: Binary Renderer, Compliance Report, IDE Extension, Config Watcher
3. **Fix 3 dependency issues**: Clear Warden interface, separate NATS subjects, DI over singletons
4. **Add 4 failure modes**: Offline mode, Local Intervention Store, POL Agent Safety Gate, Key Storage Policy
5. **Add 3 security items**: NATS ACLs, Agent sandboxing, Key rotation automation

---

*End of Reverse Audit*
