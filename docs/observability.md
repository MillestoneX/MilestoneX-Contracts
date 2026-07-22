# Observability — Diagnostic Metrics & Events

The campaign contract supports an optional `diag` feature that enables structured
tracing and runtime counters for observability. When the feature is disabled
(default), all diagnostic code is compiled away — zero storage overhead, zero
event emission.

## Feature Flag

| Flag   | Default | Description                                          |
|--------|---------|------------------------------------------------------|
| `diag` | off     | Enables diagnostic counters and `diagnostics` events |

Enable at build time:

```bash
cargo build -p milestonex-campaign --features diag --target wasm32v1-none --release
```

## Metrics View

`metrics_view` — always available; returns all-zero counters when `diag` is off.

### `CampaignMetrics`

| Counter                   | Type    | Description                               |
|---------------------------|---------|-------------------------------------------|
| `donations_total`         | `u64`   | Successful donation calls                 |
| `milestones_released_total` | `u64` | Completed milestone releases              |
| `refunds_total`           | `u64`   | Successfully processed refunds            |
| `last_diagnostics_ledger` | `u32`   | Ledger sequence of last `emit_diagnostics` |

## Diagnostics Event

`emit_diagnostics` publishes a `("campaign", "diagnostics")` event containing
the current `CampaignMetrics` struct and the ledger sequence. The event is only
emitted when the `diag` feature is enabled; when disabled the entrypoint is a
no-op.

### Event payload (feature `diag` on)

```json
{
  "topic": ["campaign", "diagnostics"],
  "data": {
    "metrics": {
      "donations_total": 42,
      "milestones_released_total": 3,
      "refunds_total": 1,
      "last_diagnostics_ledger": 20100
    },
    "ledger": 20100
  }
}
```

## Usage

```rust
// Read counters (always available)
let metrics = contract_client.metrics_view();

// Emit a diagnostics event (only emits when built with --features diag)
contract_client.emit_diagnostics();
```

## Testing

Run diagnostics tests with the default (diag off) configuration:

```bash
cargo test -p milestonex-campaign -- diagnostics
```

Run diagnostics tests with the feature enabled:

```bash
cargo test -p milestonex-campaign --features diag -- diagnostics
```
