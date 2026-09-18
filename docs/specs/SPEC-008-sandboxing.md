# SPEC-008: Sandboxing with nucleus-container

---
name: Sandboxing-nucleus
description: Multi-layered sandboxing using nucleus-container for agent execution isolation
status: draft
created: 2026-09-16
author: Lucien
related-adrs:
  - "ADR-006"
---

# Spec: Sandboxing with nucleus-container

## Goal
Implement multi-layered sandboxing using nucleus-container in strict-agent mode for secure agent execution.

## Context
Phase 2 research identified nucleus-container as the optimal sandboxing solution:
- Nix-based reproducible rootfs
- Seccomp profiles with SHA-256 pinning
- Landlock filesystem ACLs
- Fail-closed isolation
- Three modes: agent, strict-agent, production

## Requirements

### R1: Sandboxing Mode Selection

| Mode | Use Case | Isolation Level | Root Required |
|------|----------|-----------------|---------------|
| `agent` | Development/testing | Best-effort | No |
| `strict-agent` | Production agent execution | Fail-closed | Yes (or CAP_SYS_ADMIN) |
| `production` | Long-running services | Full isolation | Yes |

**Default for Horizon 1:** `strict-agent` mode

### R2: Security Controls

#### Seccomp Profile
```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "syscalls": [
    {"names": ["read", "write", "exit", "exit_group"], "action": "SCMP_ACT_ALLOW"},
    {"names": ["brk", "mmap", "munmap"], "action": "SCMP_ACT_ALLOW"},
    {"names": ["futex", "arch_prctl"], "action": "SCMP_ACT_ALLOW"},
    {"names": ["clone", "fork", "vfork"], "action": "SCMP_ACT_ERRNO"}
  ]
}
```

**Verification:**
- Profile loaded from file
- SHA-256 hash checked at runtime
- Mismatch → sandbox fails to start

#### Landlock Rules
```toml
# landlock.toml
[rules.filesystem]
read = ["/app", "/tmp/agent"]
write = ["/tmp/agent/output"]
execute = ["/usr/bin/python3", "/usr/local/bin/agent"]
```

#### Namespace Isolation
- PID namespace: processes isolated
- Mount namespace: filesystem isolated
- Network namespace: network isolated
- User namespace: UID mapping

### R3: Resource Limits

```toml
# resource_limits.toml
[memory]
limit_mb = 512
swap_limit_mb = 0

[cpu]
quota_us = 1000000  # 1 core-second per second
period_us = 1000000

[pids]
max = 100

[network]
egress_allow = ["api.openai.com", "huggingface.co"]
ingress_deny_all = true
```

### R4: Execution Flow

```
1. Agent receives spawn request
2. Policy engine evaluates (SPEC-007)
3. Warden checks security (SPEC-002)
4. nucleus-container creates sandbox:
   a. Generate seccomp profile (SHA-256 pinned)
   b. Apply Landlock rules
   c. Create namespaces
   d. Set resource limits
   e. Copy rootfs (Nix closure)
5. Execute agent binary in sandbox
6. Collect output, receipts, metrics
7. Cleanup sandbox resources
```

### R5: Testing Requirements
- [ ] Sandbox creation test
- [ ] Seccomp profile verification test
- [ ] Landlock rule test
- [ ] Resource limit enforcement test
- [ ] Escape attempt test (should fail)
- [ ] Performance benchmark

## Out of Scope
- Live migration between sandboxes
- GPU passthrough
- Custom syscall handling

## References
- [Phase 2 Research: Agent Sandboxing](../research/agent-sandboxing.md)
- [nucleus-container Documentation](https://crates.io/crates/nucleus-container)
