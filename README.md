# k8s-diff-explainer (WIP)

Explain why a Kubernetes manifest change might break a rollout. Deterministic rules, CI-friendly, no AI magic.

## Quick start

```bash
cargo run -- examples/old.yaml examples/new.yaml
```

JSON output:

```bash
cargo run -- examples/old.yaml examples/new.yaml --format json
```

Fail CI on High risk:

```bash
cargo run -- examples/old.yaml examples/new.yaml --fail-on high
```

Enable experimental rules (e.g., selector drift):

```bash
cargo run -- examples/selector-old.yaml examples/selector-new.yaml --experimental
```

## Sample output (from `examples/old.yaml` → `examples/new.yaml`)

```
Resource: Deployment/api
Namespace: prod
Overall Rollout Risk: HIGH

[HIGH] KDX001 Memory limit reduced
Container: api
Field: spec.template.spec.containers[api].resources.limits.memory
Old: 2Gi
New: 512Mi
...
[HIGH] KDX010 Replica increase with pressure signals
Field: spec.replicas
Old: 2
New: 8
```

## CLI

- `k8s-diff-explainer <old.yaml> <new.yaml>`
- `--format text|json` (default text)
- `--fail-on medium|high` sets non-zero exit for CI when overall risk meets threshold.

## Rules (v1)

- KDX001 Memory limit reduced
- KDX002 Memory request increased sharply
- KDX003 CPU reduced aggressively
- KDX004 Readiness probe became stricter
- KDX005 Liveness stricter without startupProbe
- KDX006 Image pull risk introduced
- KDX007 Required env var removed
- KDX008 Secret/ConfigMap reference changed
- KDX009 Scheduling constraints tightened
- KDX010 Replica increase with pressure signals
- KDX011 Pod selector changed (experimental flag)

## Internal model (MVP)

We normalize Deployment/StatefulSet/DaemonSet fields:

- replicas, nodeSelector, required node affinity, imagePullSecrets
- per-container: image, requests/limits, probes, env/envFrom, secret/configMap refs, ports

Rules operate on structured diffs (not raw YAML text) and produce a single `Finding` shape used by both text and JSON reports. Overall risk scoring: Low=1, Medium=3, High=6; 2+ High findings => HIGH.

## Tests

Integration smoke tests live in `tests/cli_smoke.rs` and exercise text, JSON, and fail-on behavior against the example manifests.

## Install (local)

```bash
cargo install --path .
```

## Status

This is an early WIP. The core rule set is stable and covered by smoke tests, but:
- only a subset of fields are evaluated,
- experimental rules are behind `--experimental`,
- JSON schema may still evolve before a v0.1.0 tag.
