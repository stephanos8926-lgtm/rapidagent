# ADR-006: Sandboxing Strategy

---
title: "ADR-006: Sandboxing Strategy"
status: Accepted
date: 2026-09-16
deciders: Lucien, Steven Page
consulted: RapidWebs Security Team
informed: All contributors

---

## Context

Agent execution requires isolation to prevent:
- Privilege escalation
- Resource exhaustion
- Data leakage between tenants
- Malicious code execution

Options evaluated: Docker containers, nspawn, sandbox-rs, nucleus-container.

## Decision

We will use **nucleus-container** in strict-agent mode for production agent execution.

**Decision:** Deploy nucleus-container with the following configuration:
- Mode: `strict-agent` (fail-closed)
- Seccomp: Custom JSON profile with SHA-256 pinning
- Landlock: Filesystem ACLs for agent workspace
- Namespaces: PID, mount, network, user
- Resource limits: 512MB RAM, 1 CPU, 100 PIDs

## Alternatives Considered

| Option | Pros | Cons | Reason Rejected |
|--------|------|------|-----------------|
| **Docker** | Mature, well-documented | Heavy, requires daemon, slower startup | Overkill for single-agent execution |
| **nspawn** | Lightweight, systemd-integrated | Limited seccomp support, no Landlock | Insufficient isolation |
| **sandbox-rs** | Pure Rust, unprivileged | Limited namespace support, no Landlock | Weaker isolation than nucleus |
| **nucleus-container** ✅ | Nix reproducible, full namespace support, Landlock, seccomp pinning | Requires root for strict mode, newer crate | Best security posture |

## Consequences

### Positive
- **Fail-closed**: Sandbox fails to start if security check fails
- **Reproducible**: Nix-based rootfs ensures consistency
- **Fine-grained control**: Per-syscall seccomp, path-based Landlock
- **Resource isolation**: Cgroups v2 for memory, CPU, PID limits
- **Network isolation**: egress allowlist, ingress deny-all

### Negative
- **Root required**: Strict mode needs root or CAP_SYS_ADMIN
- **Startup overhead**: ~100ms for sandbox creation
- **Nix dependency**: Requires Nix package manager
- **Complexity**: More configuration than Docker

### Mitigations
- Run as privileged service account (not root)
- Optimize sandbox reuse for repeated executions
- Document Nix setup in onboarding guide

## Implementation Notes

- Use `nucleus-container` crate v0.5+
- Store seccomp profiles in Git for version control
- Implement profile verification at runtime
- Test escape attempts regularly

## References
- [SPEC-008: Sandboxing](../specs/SPEC-008-sandboxing.md)
- [nucleus-container Documentation](https://crates.io/crates/nucleus-container)
