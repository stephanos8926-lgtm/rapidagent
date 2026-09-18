# Research: Agent Sandboxing and Security Isolation

**Date**: 2026-09-15  
**Author**: Lucien (Lead Digital Architect)  
**Status**: Complete  

---

## Executive Summary

Multi-layered sandboxing is essential for safe agent execution. This research examines three approaches: kernel primitives (seccomp + Landlock + namespaces), container runtimes (nucleus-container), and isolated execution environments. Each offers different trade-offs between security, performance, and complexity.

---

## Approach 1: Kernel Primitives (Unprivileged)

### Sandbox-RS Library

**Status**: Production-ready  
**Requirements**: Linux kernel 5.10+, optional root for full isolation

#### Features

| Feature | Implementation | Security Level |
|---------|---------------|----------------|
| **Seccomp BPF** | Six profiles: Essential, Minimal, IoHeavy, Compute, Network, Unrestricted | Process-level syscall filtering |
| **Landlock** | Filesystem access control (Linux 5.13+) | Filesystem-level isolation |
| **Namespaces** | User, mount, PID, network namespaces | Process and filesystem isolation |
| **Cgroups** | Memory, CPU, PID limits | Resource constraints |
| **setrlimit** | Additional resource limits | Fine-tuned constraints |

#### Example Usage

```rust
use sandbox_rs::{SandboxBuilder, SeccompProfile, PrivilegeMode};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sandbox = SandboxBuilder::new("agent-sandbox")
        .privilege_mode(PrivilegeMode::Unprivileged)
        .memory_limit_str("256M")?
        .cpu_limit_percent(50)
        .timeout(Duration::from_secs(30))
        .seccomp_profile(SeccompProfile::IoHeavy)
        .build()?;

    let result = sandbox.run("python3", &["script.py"])?;
    println!("exit={} mem={}B cpu={}μs", 
        result.exit_code, result.memory_peak, result.cpu_time_us);
    
    Ok(())
}
```

#### Seccomp Profiles

| Profile | Syscalls | Use Case |
|---------|----------|----------|
| `Essential` | ~40 (execve, mmap, brk, read, write, exit) | Bootstrap only |
| `Minimal` | ~110 (Essential + signals, pipes, timers) | Basic execution |
| `IoHeavy` | Minimal + file ops (mkdir, chmod, unlink, rename) | File manipulation |
| `Compute` | IoHeavy + scheduling (sched_setscheduler, mbind) | CPU-intensive tasks |
| `Network` | Compute + sockets (socket, bind, listen, connect) | Network access |
| `Unrestricted` | Network + privileged (ptrace, mount, bpf) | Debugging only |

---

## Approach 2: nucleus-container (Recommended)

**Status**: Production-ready  
**License**: Apache-2.0  
**Rootfs**: Nix-based reproducible environments

### Three Execution Modes

| Mode | Use Case | Isolation | Requirements |
|------|----------|-----------|--------------|
| **Agent** | Ephemeral AI workloads | Best-effort | None |
| **Strict-Agent** | Security-critical ephemeral | Fail-closed | Root or CAP_SYS_ADMIN |
| **Production** | Long-running services | Full isolation | NixOS rootfs |

### Agent Mode Configuration

```bash
nucleus run \
  --service-mode agent \
  --memory 512M \
  --cpus 1 \
  --workspace /path/to/workspace \
  -- python3 script.py
```

### Strict-Agent Mode (Recommended for RapidAgent)

```bash
nucleus run \
  --service-mode strict-agent \
  --memory 512M \
  --cpus 1 \
  --seccomp-profile ./config/agent.seccomp.json \
  --seccomp-profile-sha256 "$(sha256sum config/agent.seccomp.json | cut -d' ' -f1)" \
  --landlock-policy ./config/agent.landlock.toml \
  -- /path/to/agent-binary
```

### Security Features

| Feature | Implementation | Enforcement |
|---------|---------------|-------------|
| **Capabilities** | All dropped by default | Irreversible via TOML policy |
| **Seccomp** | Per-service JSON profiles with SHA-256 pinning | Irreversible |
| **Landlock** | Path-based filesystem ACLs | Linux 5.13+ required |
| **Namespaces** | Up to 8 namespace types | Full isolation in strict mode |
| **Network** | Deny-by-default with per-CIDR/domain allowlist | iptables-based egress control |
| **Rootfs** | Optional Nix closure with attestation | Read-only by default |

### Seccomp Profile Generation

```bash
# 1. Trace mode records all syscalls
nucleus run \
  --seccomp-mode trace \
  --seccomp-log ./trace.ndjson \
  --memory 512M \
  -- /bin/my-service

# 2. Generate minimal profile from trace
nucleus seccomp generate ./trace.ndjson -o config/agent.seccomp.json

# 3. Verify and enforce
nucleus run \
  --seccomp-profile ./config/agent.seccomp.json \
  --seccomp-profile-sha256 "$(sha256sum config/agent.seccomp.json | cut -d' ' -f1)" \
  -- /bin/my-service
```

---

## Approach 3: OCI Containers with crun

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Container Runtime                         │
├─────────────────────────────────────────────────────────────┤
│  OCI Bundle (config.json + rootfs/)                          │
│  ├── Linux Namespaces (PID, NET, MNT, IPC, USER)            │
│  ├── Seccomp Filter (syscall allowlist)                      │
│  ├── AppArmor Profile (MAC)                                  │
│  └── Cgroup (resource limits)                                │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                    crun / runc                               │
│  - Creates namespaces                                       │
│  - Applies seccomp filter                                   │
│  - Sets up cgroup                                           │
│  - Executes container process                               │
└─────────────────────────────────────────────────────────────┘
```

### Implementation from ssamd

```rust
// ContainerBundleSpec: inputs for OCI bundle
pub struct ContainerBundleSpec {
    oci_template: Spec,                    // Base OCI template
    seccomp_policy: Option<Vec<u8>>,       // Syscall filter policy
    mac_enabled: bool,                     // AppArmor/SELinux
    network_mode: NetworkMode,             // Host/None/Bridge
    cgroups_path: String,                  // Resource limits
    package_name: String,                  // Container ID
    mount_point: PathBuf,                  // Rootfs mount
    data_mounts: Option<DataMountPaths>,   // Bind mounts
    bridge_netns_path: Option<PathBuf>,    // Network namespace
}

// TransientRuntimeConfig: physical OCI bundle
pub struct TransientRuntimeConfig {
    temp_dir: TempDir,                     // /tmp/XXXXXX with 0700 perms
    bundle_path: PathBuf,                  // Bundle directory
}
```

### Security Pipeline

1. **Create TempDir** with 0700 permissions (atomic, non-world-readable)
2. **Generate config.json** from template + security policies
3. **Apply seccomp filter** from JSON policy
4. **Configure namespaces** (PID, network, mount, IPC)
5. **Set AppArmor profile** (container-default or package-specific)
6. **Configure cgroups** path for resource limits
7. **Execute via crun** with bundle path

---

## Approach Comparison

| Feature | Kernel Primitives | nucleus-container | OCI Containers |
|---------|------------------|-------------------|----------------|
| **Isolation Level** | Medium | High (strict-agent) | High |
| **Root Required** | No (unprivileged mode) | Yes (strict mode) | Yes |
| **Startup Time** | < 10ms | ~100ms | ~500ms |
| **Overhead** | Minimal | Low | Moderate |
| **Filesystem ACLs** | Landlock (Linux 5.13+) | Landlock + Rootfs | None (by default) |
| **Network Control** | Namespace | iptables egress | Network namespace |
| **Resource Limits** | Cgroups | Cgroups v2 | Cgroups v2 |
| **Reproducibility** | None | Nix closures | Docker images |
| **Complexity** | Low | Medium | High |

---

## Application to RapidAgent

### Recommended Strategy

| Component | Sandboxing | Rationale |
|-----------|------------|-----------|
| **Agent execution** | `nucleus-container` strict-agent | Balances security + performance |
| **Tool calls** | Kernel primitives (sandbox-rs) | Lightweight per-tool isolation |
| **Network access** | n
