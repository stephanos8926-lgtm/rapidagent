# SPEC-007: OPA Policy Engine Integration

---
name: OPA-Policy-Engine
description: Embedded OPA/Rego policy engine for agent governance
status: draft
created: 2026-09-16
author: Lucien
related-adrs:
  - "ADR-008"
---

# Spec: OPA Policy Engine Integration

## Goal
Integrate embedded OPA/Rego policy engine for declarative agent governance with bundle distribution and unit testing.

## Context
Phase 2 research established embedded OPA as optimal for low-latency policy evaluation:
- <1ms p99 latency for simple policies
- Declarative Rego language
- Bundle-based policy distribution
- Built-in unit testing framework

## Requirements

### R1: Policy Structure

```rego
# Policy: agent_spawn.rego
package rapidagent.policies.spawn

import input as event

# Allow spawn if user has permission
allow {
    event.auth.role == "admin"
}

allow {
    event.auth.role == "developer"
    event.auth.permissions[_] == "agent.spawn"
}

# Rate limiting
deny[msg] {
    count[event.type == "spawn"] > 100
    msg := "Rate limit exceeded: max 100 spawns per minute"
}
```

### R2: Policy Bundle Format

```
bundle/
├── policy/
│   ├── agent_spawn.rego
│   ├── agent_stop.rego
│   ├── tool_access.rego
│   └── data_quota.rego
├── metadata.json
└── revision.txt
```

**metadata.json:**
```json
{
  "manifest": {
    "files": [
      {"name": "policy/agent_spawn.rego", "sha256": "..."},
      {"name": "policy/agent_stop.rego", "sha256": "..."}
    ]
  },
  "revision": "20260916-001"
}
```

### R3: Runtime Integration

```rust
use opa_wasm::Runtime;

struct PolicyEngine {
    runtime: Runtime,
    bundle_path: PathBuf,
    cache: LruCache<String, bool>,
}

impl PolicyEngine {
    fn evaluate(&mut self, policy: &str, input: &Value) -> Result<bool> {
        // Check cache first
        let cache_key = format!("{}:{}", policy, input);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(*cached);
        }
        
        // Evaluate policy
        let result = self.runtime.eval(policy, input)?;
        
        // Cache result (TTL 60s)
        self.cache.insert(cache_key, result);
        
        Ok(result)
    }
}
```

### R4: Policy Evaluation Points

| Point | Policy Package | Input Type |
|-------|---------------|------------|
| Agent spawn | `rapidagent.policies.spawn` | SpawnRequest |
| Tool access | `rapidagent.policies.tool_access` | ToolCall |
| Data quota | `rapidagent.policies.data_quota` | QuotaRequest |
| Agent stop | `rapidagent.policies.stop` | StopRequest |
| Intervention | `rapidagent.policies.intervention` | InterventionRequest |

### R5: Testing Requirements
- [ ] Unit tests for each policy
- [ ] Bundle loading test
- [ ] Cache invalidation test
- [ ] Performance benchmark (<1ms p99)
- [ ] Policy update hot-reload test

## Out of Scope
- WASM policy compilation
- Policy versioning UI
- Policy conflict detection

## References
- [Phase 2 Research: Policy as Code](../research/policy-as-code-opa.md)
- [OPA Documentation](https://www.openpolicyagent.org/docs/)
