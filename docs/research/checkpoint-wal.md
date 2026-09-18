# Research: Agent Checkpoint and Write-Ahead Logging

**Date**: 2026-09-15  
**Author**: Lucien (Lead Digital Architect)  
**Status**: Complete  

---

## Executive Summary

Checkpoint and WAL (Write-Ahead Logging) mechanisms are critical for agent crash recovery. This research examines SQLite WAL mode, custom checkpoint strategies, and fault-tolerant storage patterns for RapidAgent's state management.

---

## SQLite WAL Mode

### Overview

SQLite's Write-Ahead Logging (WAL) mode provides concurrent read/write access without blocking readers. Committed transactions are appended to a WAL file, then checkpointed back to the main database.

### Checkpoint Modes

| Mode | Behavior | Use Case |
|------|----------|----------|
| **PASSIVE** | Checkpoints without blocking | Background maintenance |
| **FULL** | Blocks writers until complete | Critical checkpoints |
| **RESTART** | FULL + resets WAL file | Periodic cleanup |
| **TRUNCATE** | RESTART + truncates WAL | Aggressive space reclamation |

### Implementation

```rust
use rusqlite::{Connection, OpenFlags};
use rusqlite::hooks::{HookResult, Wal};

fn create_checkpoint_hook(conn: &Connection) -> anyhow::Result<()> {
    conn.add_hook(Wal::new(
        "main",
        move |_wal, _db| {
            // Trigger checkpoint on every commit
            Ok(HookResult::Action)
        },
    ))?;
    
    // Enable WAL mode
    conn.execute("PRAGMA journal_mode=WAL", [])?;
    
    // Set auto-checkpoint to 1000 pages
    conn.execute("PRAGMA wal_autocheckpoint=1000", [])?;
    
    Ok(())
}
```

### Checkpoint Monitoring

```rust
use rusqlite::hooks::Wal;
use rusqlite::types::ValueType;

fn get_checkpoint_stats(conn: &Connection) -> anyhow::Result<(i32, i32)> {
    let (log_size, checkpointed) = conn.with_transaction(|txn| {
        txn.pragma_query_value("wal_checkpoint", [], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
            ))
        })?
    })?;
    
    Ok((log_size, checkpointed))
}
```

---

## Custom Checkpoint Strategy

### Agent State Checkpoint

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCheckpoint {
    pub agent_id: String,
    pub tenant_id: String,
    pub turn_count: u64,
    pub state_snapshot: HashMap<String, serde_json::Value>,
    pub graph_position: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
}

impl AgentCheckpoint {
    pub fn compute_checksum(&self) -> String {
        use sha2::{Sha256, Digest};
        let content = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub fn validate(&self) -> bool {
        self.compute_checksum() == self.checksum
    }
}

pub struct CheckpointManager {
    db: Connection,
    checkpoint_dir: PathBuf,
}

impl CheckpointManager {
    pub fn save(&self, checkpoint: &AgentCheckpoint) -> anyhow::Result<PathBuf> {
        // Validate checksum
        assert!(checkpoint.validate(), "Checkpoint checksum mismatch");
        
        // Save to SQLite (WAL mode)
        self.db.execute(
            "INSERT INTO checkpoints (agent_id, tenant_id, turn_count, state, graph_pos, created_at, checksum)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                &checkpoint.agent_id,
                &checkpoint.tenant_id,
                checkpoint.turn_count,
                serde_json::to_string(&checkpoint.state_snapshot)?,
                &checkpoint.graph_position,
                checkpoint.created_at,
                &checkpoint.checksum,
            ],
        )?;
        
        // Also save to filesystem for portability
        let checkpoint_path = self.checkpoint_dir.join(format!(
            "{}_t{}.json",
            checkpoint.agent_id, checkpoint.turn_count
        ));
        
        std::fs::write(
            &checkpoint_path,
            serde_json::to_string_pretty(checkpoint)?,
        )?;
        
        Ok(checkpoint_path)
    }
    
    pub fn load_latest(&self, agent_id: &str) -> anyhow::Result<Option<AgentCheckpoint>> {
        use rusqlite::OptionalExtension;
        
        let row = self.db.query_row(
            "SELECT agent_id, tenant_id, turn_count, state, graph_pos, created_at, checksum
             FROM checkpoints
             WHERE agent_id = ?
             ORDER BY turn_count DESC
             LIMIT 1",
            rusqlite::params![agent_id],
            |row| {
                Ok(AgentCheckpoint {
                    agent_id: row.get(0)?,
                    tenant_id: row.get(1)?,
                    turn_count: row.get(2)?,
                    state_snapshot: serde_json::from_str(row.get(3)?)?,
                    graph_position: row.get(4)?,
                    created_at: row.get(5)?,
                    checksum: row.get(6)?,
                })
            },
        ).optional()?;
        
        Ok(row)
    }
}
```

---

## Crash Recovery Flow

### Recovery Sequence

```
┌─────────────────────────────────────────────────────────────┐
│                      Agent Startup                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  1. Check SQLite WAL for uncommitted transactions           │
│     - SQLite automatically recovers on open                 │
│     - WAL file provides crash consistency                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  2. Load latest checkpoint from filesystem                  │
│     - Verify checksum integrity                             │
│     - Validate checkpoint timestamp                         │
└─────────────────────────────────────────────────────────────┐
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  3. Replay transactions from WAL after checkpoint           │
│     - Apply any in-flight turns                             │
│     - Resolve partial executions                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  4. Resume from recovered state                             │
│     - Restore graph position                                │
│     - Re-establish NATS subscriptions                       │
│     - Notify POL of recovery                                │
└─────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
pub async fn recover_agent(
    agent_id: &str,
    checkpoint_mgr: &CheckpointManager,
    nats: &NatsConnection,
    pol_client: &PolClient,
) -> anyhow::Result<RecoveredAgent> {
    // Step 1: SQLite WAL recovery is automatic on connection open
    let db = Connection::open_with_flags(
        format!("/var/lib/rapidagent/checkpoints/{}.db", agent_id),
        OpenFlags::SQLITE_OPEN_READ_WRITE | 
        OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    
    // Step 2: Load latest valid checkpoint
    let checkpoint = checkpoint_mgr.load_latest(agent_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No checkpoint found"))?;
    
    // Validate checksum
    if !checkpoint.validate() {
        tracing::error!("Checkpoint checksum invalid, rolling back");
        return Err(anyhow::anyhow!("Checkpoint corruption detected"));
    }
    
    // Step 3: Replay any pending transactions
    let pending_turns = replay_pending_transactions(&db, checkpoint.turn_count).await?;
    
    // Step 4: Re-establish context
    Ok(RecoveredAgent {
        agent_id: agent_id.to_string(),
        state: checkpoint.state_snapshot,
        graph_position: checkpoint.graph_position,
        turn_count: checkpoint.turn_count,
        pending_turns,
    })
}
```

---

## Advanced: fssqlite-wal

For production-grade WAL management, consider the `fsqlite-wal` crate which provides:

- **Frame checksumming** with XXH3-128
- **Torn-write detection** using sector-size analysis
- **Forward error correction** via RaptorQ erasure coding
- **Checkpoint planning** with deterministic mode selection

### Example Usage

```rust
use fsqlite_wal::{WalHeader, CheckpointMode};

async fn advanced_checkpoint(db: &Connection) -> anyhow::Result<()> {
    // Plan checkpoint
    let state = db.wal_state()?;
    let plan = fsqlite_wal::plan_checkpoint(CheckpointMode::Restart, state);
    
    // Execute checkpoint
    let progress = fsqlite_wal::execute_checkpoint(db, plan).await?;
    
    tracing::info!(
        "Checkpoint progress: {:.1}%",
        progress.completion() * 100.0
    );
    
    Ok(())
}
```

---

## Best Practices

### 1. Checkpoint Frequency

| Scenario | Checkpoint Interval | Rationale |
|----------|---------------------|-----------|
| Development | Every 10 turns | Fast iteration |
| Production (critical) | Every 5 turns | Minimize recovery window |
| Production (standard) | Every 30 turns | Balance I/O overhead |
| Batch processing | After each batch | Natural boundary |

### 2. Storage Layout

```
/var/lib/rapidagent/
├── checkpoints/
│   ├── {tenant_id}/
│   │   ├── {agent_id}.db           # SQLite database (WAL mode)
│   │   ├── {agent_id}.db-wal       # WAL file
│   │   ├── {agent_id}_t0.json      # Checkpoint at turn 0
│   │   ├── {agent_id}_t30.json     # Checkpoint at turn 30
│   │   └── {agent_id}_t60.json     # Checkpoint at turn 60
│   └── ...
└── wal/
    └── {agent_id}.log              # Application-level WAL
```

### 3. Integrity Verification

```rust
fn verify_checkpoint_integrity(path: &Path) -> bool {
    // 1. Check file exists
    if !path.exists() {
        return false;
    }
    
    // 2. Read and parse JSON
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    
    let checkpoint: AgentCheckpoint = match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(_) => return false,
    };
    
    // 3. Validate checksum
    checkpoint.validate()
}
```

---

## References

- [SQLite WAL Documentation](https://www.sqlite.org/wal.html)
- [rusqlite Hooks](https://docs.rs/rusqlite/latest/rusqlite/hooks/)
- [fssqlite-wal Crate](https://docs.rs/crate/fsqlite-wal)

---

*Research completed: 2026-09-15*  
*Next steps: Implement CheckpointManager in rapidagent-engine crate*
