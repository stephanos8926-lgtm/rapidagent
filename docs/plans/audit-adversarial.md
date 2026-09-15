# Adversarial Audit: RapidAgent Framework Architecture

## Scope
- Attack the architecture: find failure modes, exploit assumptions, stress-test decisions
- Focus on: worst-case scenarios, single points of failure, security vulnerabilities

---

## 🚨 CRITICAL VULNERABILITIES

### 1. NATS Single Point of Failure
**Attack Vector**: NATS cluster goes down → entire platform halts
**Impact**: Total service outage
**Current Mitigation**: None explicit
**Recommendation**: 
- Implement `INF-007`: NATS HA cluster (3+ nodes)
- Add `ENG-008`: Offline mode with local event queue
- Local queue persists events → replays on reconnect

### 2. PostgreSQL Single Point of Failure
**Attack Vector**: PG crashes → interventions lost, audit trail broken
**Impact**: Data loss, compliance violation
**Current Mitigation**: None
**Recommendation**:
- Add `INF-008`: PG replica (streaming replication)
- Add `POL-009`: Local intervention journal (append-only JSON, sync to PG)

### 3. Warden Blocking Deadlock
**Attack Vector**: Warden process hangs → all agent tool calls block forever
**Impact**: Complete agent freeze
**Current Mitigation**: Fail-closed (timeout = deny)
**Gap**: First-boot grace period undefined
**Recommendation**:
- Explicit grace period config: `warden.grace_period_seconds = 30`
- Health probe that marks Warden unhealthy if no responses in 5s
- Circuit breaker on Warden calls

---

## 🔴 ARCHITECTURAL WEAKNESSES

### 4. Monolithic Daemon
**Problem**: `DRN-001` hosts POL + Warden + Spawn Manager
**Risk**: Any component crash takes down everything
**Mitigation**: 
- Split into separate processes (POL daemon, Warden daemon, Spawn daemon)
- OR add strict process isolation via systemd slices

### 5. No Version Pinning Between Crates
**Problem**: `rapidagent-engine` v0.3.1 may break with `rapidagent-ir` v0.4.0
**Risk**: Dependency hell, silent incompatibilities
**Mitigation**:
- Add `CMP-007`: Version Compatibility Checker
- Enforce semver across workspace
- CI job that tests cross-crate compatibility matrix

### 6. Event Schema Drift
**Problem**: `FWD-002` EHP Envelope v1 evolves → old agents break
**Risk**: Breaking changes in production
**Mitigation**:
- Schema registry with versioning
- Backward compatibility tests in CI
- Deprecation warnings for old envelope versions

---

## 🟡 SECURITY CONCERNS

### 7. Ed25519 Key Storage
**Problem**: Signing keys stored in filesystem or env vars
**Risk**: Key theft → forged agent artifacts
**Mitigation**:
- Add `SEC-005`: Key Storage Policy (HSM/Vault required)
- Key rotation automation (`SEC-003` → `SEC-009`)

### 8. NATS Auth Bypass
**Problem**: NATS ACLs not enforced → unauthenticated event injection
**Risk**: Attackers inject false POL interventions
**Mitigation**:
- Add `SEC-006`: NATS Security Configuration
- Per-subject ACLs, JWT auth, TLS mandatory

### 9. Agent Sandbox Escape
**Problem**: Untrusted agents access host filesystem/network
**Risk**: Arbitrary code execution on host
**Mitigation**:
- Add `SEC-007`: Seccomp/AppArmor Profiles
- Network namespace isolation per agent
- Filesystem jailing via chroot/nspawn

---

## 🔥 OPERATIONAL NIGHTMARES

### 10. Deployment Orchestration
**Problem**: 62 components across 3 VMs, manual deployment
**Risk**: Inconsistent state, failed updates
**Mitigation**:
- Add `CD-005`: Blue-Green Deploy Script
- Add `CD-006`: Rolling Update Manager
- Ansible playbook for infrastructure (`CD-003`)

### 11. Debugging Distributed System
**Problem**: No centralized tracing across Rust/Python/TS
**Risk**: Hours spent correlating logs
**Mitigation**:
- Add `OBS-005`: Trace Correlation Engine (OTel collector)
- Standardized span attributes across all languages
- Jaeger/Tempo dashboard

### 12. Memory Leak in Long-Running Agents
**Problem**: Engine instances run for days/weeks
**Risk**: OOM kills, state corruption
**Mitigation**:
- Memory profiling in tests
- Max session duration config
- Automatic checkpoint + restart on memory threshold

---

## 💀 WORST-CASE SCENARIOS

### Scenario A: POL Agent Hallucination Cascade
1. POL Agent analyzes agent session, hallucinates "fix"
2. POL injects malicious guidance into 100+ agents
3. Agents execute destructive commands
4. **Mitigation**: Rules-first POL Agent (never trust LLM output), human gate on critical actions

### Scenario B: Warden Race Condition
1. Two agents request same resource simultaneously
2. Warden evaluates both before either responds
3. Both allowed → data corruption
4. **Mitigation**: Distributed locking via NATS KV, or sequential Warden processing

### Scenario C: Compiler/SIGVUL
1. Malicious `.rag` file crafted to exploit compiler
2. Buffer overflow in Rust parser (if unsafe code)
3. Remote code execution on compilation server
4. **Mitigation**: Fuzz testing, miri checks, no unsafe code in parser, sandboxed compilation

---

## ✅ RECOMMENDED ADDITIONS

| ID | Component | Purpose |
|----|-----------|---------|
| `INF-007` | NATS HA Cluster | Eliminate NATS SPOF |
| `INF-008` | PG Replica | Eliminate PG SPOF |
| `POL-009` | Local Intervention Journal | Survive PG outage |
| `SEC-005` | Key Storage Policy | Secure signing keys |
| `SEC-006` | NATS Security Config | Prevent auth bypass |
| `SEC-007` | Agent Sandboxing | Prevent escape |
| `SEC-009` | Key Rotation Automation | Rotate signing keys |
| `CD-005` | Blue-Green Deploy | Zero-downtime updates |
| `CD-006` | Rolling Update Manager | Safe multi-VM deploys |
| `OBS-005` | Trace Correlation | Cross-component debugging |

---

*End of Adversarial Audit*
