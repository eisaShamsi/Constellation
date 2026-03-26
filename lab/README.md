# Constellation Lab

## Experiment Lab + Audit System

This directory contains the testing, experimentation, and audit infrastructure for building eNotePane — the ultimate note.

### Structure

```
lab/
  experiments/    — isolated test components (one feature per experiment)
  benchmarks/     — performance measurement scripts
  reports/        — audit results and experiment logs
  README.md       — this file
```

### Workflow

1. **Propose** a feature or change
2. **Experiment** in `lab/experiments/` — isolated, no production impact
3. **Benchmark** with `lab/benchmarks/` — measure typing latency
4. **Audit** with agents — unbiased validation against eNotePane spec
5. **Approve** only if all auditors pass
6. **Implement** in production `src/`

### Rule: Nothing enters production without passing the lab.
