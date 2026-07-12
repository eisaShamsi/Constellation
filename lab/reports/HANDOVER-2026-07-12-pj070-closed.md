# Handover — 2026-07-12 — PJ-070 CLOSED (watcher external-change adopt) · backlog → v1.19

**Read `docs/Constellation Orientation & Onboarding v3.40.md` first** (highest version). Then this. Then `git pull origin main` and `git log --oneline -10`.

---

## 1. What shipped this session (verified + protected)

- **PJ-070 — the watcher external-change APP-KILLER — SHIPPED + Boss-validated + `/migration` CLOSED.** An external `.md` edit to an OPEN note is no longer silently clobbered by the next keystroke — it's adopted into the single-ownership model (clean → adopt + remount; unsaved-conflict → a `.conflict` side-copy + banner, **zero loss**). Behind `WATCHER_ADOPT_ENABLED` (rollback lever). Commits **`b1a3e388`** (§1–§6) + **`cd5e53fd`** (audit hardening). Architect+Plan+reproduction docs under `docs/PJ-070-*` + `lab/reports/PJ-070-reproduction.md`.
- **The class fix (important):** a merely-VIEWED note no longer spuriously marks itself dirty (`setBody` string-no-op) — this had silently defeated the adopt on background/focus tabs + churned untouched notes on universe switch. Closed at the source.
- **Docs (SO close):** orientation **v3.40**, session log `SESSION-LOG-2026-07-12.md`, Pending Jobs **v1.19**, User Manual + 15-locale help, Charter register, MoCh `MoCh-2026-07-12-1150.md`.

## 2. State of standing

- **Verified-shipped + protected:** everything in §1. svelte-check **0**, vitest **334**, cargo **clean**. Release binary `src-tauri/target/release/constellation.exe` @ **2026-07-12 14:34** (frontend rebuilt first).
- **At-risk / uncommitted:** none after the close commit lands.
- **Known-broken:** none new. The 14 pre-existing safety-inspection findings are filed (PJ-085/086/087 + mapped to PJ-074/075 + a Group-4 batch) — none block anything.
- **PJ-072 lead:** the active "Eisa Cognitive Knowledge" universe root = **`E:\Cognitive Knowledge\`** (write-journal-confirmed). Diagnostic build still wanted for WHERE the name→root mapping persists.

## 3. NEXT — Boss ruling holds: Group 1, top-down. Next item = PJ-071.

**► Next action — PJ-071** (bulk Accept-All unlocked read-modify-write race): the proven `gate_rmw` pattern already exists next door (per-card accept was migrated; bulk wasn't) — a concurrent editor save in the window is silently overwritten. `sources/bulk_ops.rs:305`. Reproduce-First → likely a focused fix (not a full `/migration` — it's a single write path), but confirm the scope against the existing `gate_rmw` call sites.

Group-1 order after PJ-071 (from Pending Jobs v1.19): **PJ-086** (switchTab flush gap, HIGH) → **PJ-085 + PJ-073** (frontmatter/YAML, HIGH) → **PJ-074** (durable rename + cascades) → **PJ-083** (cascade sync-clear hazard) → **PJ-087** (universe.rs tmp race) → PJ-075/076/072/002.

## 4. Standing rules that bit here (carry forward)

- **SO#9:** the PJ ledger (`v1.19`) is the living backlog — reconcile it FIRST at every job-close, in the same commit as the work.
- **Reproduce-First (top principal):** no editor-lifecycle fix ships before the bug is reproduced on demand. `svelte-check`/`vitest` are NOT runtime verification for this class — the running-app Boss test is.
- **Every build:** diff-scoped `safety-inspection`; per-cycle whole-app sweep at the migration/session close; `/simplify`; svelte-check 0; `npm run build` BEFORE `cargo build --release`; verify binary mtime.
- **Solve-the-Class:** the content-integrity class is fixed structurally (the `setBody` no-op was the right depth over the per-surface `handleFlush` gate) — never a per-surface patch.

## 5. Where to resume

`lab/reports/NEXT-SESSION-PROMPT.md` is the ready-to-paste kickoff. Full narrative: `SESSION-LOG-2026-07-12.md`. Conversational trace: `docs/MoCh/MoCh-2026-07-12-1150.md`. Backlog: `docs/Constellation Pending Jobs v1.19.md`. Charter register: `docs/Constellation-Safety-Audit-CHARTER.md` (PJ-070 close cycle).
