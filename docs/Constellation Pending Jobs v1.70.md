# Constellation Pending Jobs

**Version 1.70 | 2026-08-08**

> **What changed in v1.70** (**PJ-207 §10 built and Boss-passed — one progress strip, three consumers planned, one implementation. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §11, THE DOOR** — *Repair index in Settings, Repair now on the drift notice, and every surface refreshes.* The step that finally makes the repair reachable; PJ-223's 825 missing notes are its first customer. Carry v1.69's §11 prep notes into the build: (a) the repair needs a **recompute** path for the drift report after a run — clearing the notice on `ok:true` is a success CLAIM, the class this migration exists to end; the `scan()`/`heal()` split of `reconcile::run` is the shape; (b) extract the drift listener + race guard into a `.svelte.ts` store BEFORE the second screen duplicates it; (c) the plan's §11 text anchors "Repair now" on the `storeHealthError` bar — §9 deliberately did not put drift there; the natural host is the drift notice band; reconcile the plan text first; (d) §10's core header documents the strip seam: `index_repair_status` plugs in as-is, the progress EVENT (JobProgressEvent shape, emit site `note_progress`) is §11's Rust work, plus `indexRepair.*` labels ×15. Then **§12** docs · **§13 gated on PJ-224 (Boss ruling)** · **§14** flag-off re-read · **§15** close.
>
> ### ✅ PJ-207 §10 — BUILT, Boss-passed 2026-08-08
> Two byte-equivalent 159-line strips → `JobProgressStrip.svelte` + `jobProgressCore.ts` (plain-TS state machine, 9 vitest pins — the repo has no component-mount harness, so the decisions moved to where vitest reaches). Clones deleted, consumers re-pointed, `MigrationProgressStrip` stays (different contract). Fixed while here (WA#6): a mount/destroy listener race both originals carried; the dead `$t(k) || 'fallback'` idiom; `$state.raw` for the replace-only snapshot. Diff inspection **0 confirmed**. Boss test: Stage 1 appear/count with the digit-exact predicted total (721); Stage 2 cancel + 4 s linger on the summary build (live 1,568 vs 1,619 measured — allowance stated in the test). Pipeline: five rounds, every rejection correct — two were MY overstatements (".md never touched" → the summariser READS them; "every background job" → migrations keep their own strip). Recover-on-mount stays unit-pinned; its live demonstration lands with §11's long-running repair.
>
> ### 🧰 Session-tooling note (not a Constellation PJ)
> Agent task `.output` files are left EMPTY by the harness for subagents — a draft "at that path" inspects as 0 bytes. Standing practice now: write every pipeline draft to scratchpad explicitly before gating.
>
> ### 📌 STILL OPEN, unchanged from v1.69
> PJ-224 gates §13 (Boss ruling required) · PJ-223 (reported by §9; §11 fixes) · PJ-219 design ruling (+ the federated-drift asymmetry note) · PJ-225 mtime sweep · PJ-226 walker-classification sweep (20×) · PJ-227 linked-universe phantom rows · PJ-220 (`{name:}` form + args delivery still open; CRLF proven for scriptPath) · PJ-221 `bases.rs:796` APP-KILLER · PJ-222 · `store.ts loadWorkspaces` APP-KILLER · the 2026-07-30 25 lost candidates · the 38-finding register (`wbxz23bdr`) · PJ-172 Sight flakes.
>
> **Gates:** vitest 909/909 · svelte-check 0 · i18n 15/15 ✓ · Rust untouched by §10 (1370/0 standing).
>
> ---

**Version 1.69 | 2026-08-08**

> *(See `Constellation Pending Jobs v1.69.md` — the trail is durable, never overwritten.)*
