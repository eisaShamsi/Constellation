Read docs/HANDOVER-2026-08-07-pj207.md first, then
docs/Constellation Orientation & Onboarding v3.84.md,
docs/Constellation Pending Jobs v1.68.md, and docs/PJ-207-Index-Repair-Plan.md.

Working on: PJ-207 §9 — Constellation notices, after it opens, that notes changed
while it was closed.

Context: PJ-207 is a 15-step /migration. §1–§8 are shipped, pushed and Boss-tested
(head 57144760). §9–§15 are unstarted and their plan is unchanged. §11 is the door —
the step that finally makes the repair reachable.

§9 builds the post-paint, stat-only drift check: "Criterion 4", specified 2026-04-15
in lab/boot-perf/BOOT-BUDGET.md and never built. It reads no file bytes and writes no
row, measured at 160–590 ms, and it is NOT the walk the ZERO-BOOT-WALKS rule bans. It
must take its roots from the OWN library set §8 established (libraries::try_load_libraries),
not the federation-recursive one. Per the plan's own rule, §9 carries its own 15 locale
strings in the same commit — an en-only key means 14 locales short, parity exit 1, vitest red.

Two things §8 discovered that change what comes after it. Read them before planning:

1. PJ-224 BLOCKS §13. Constellation's ordinary search box does not reach linked
   universes at all — a plain word routes to execute_universal_search, which queries the
   active universe's connection alone, while federated_lexical_search_or_fallback is
   reachable only from the advanced-syntax branches. §8's premise "foreign notes stay
   searchable via federation" is true of that command and false of the box the user types
   into. §13 offers to DELETE the duplicated copies on that premise. Do not start §13
   until the Boss rules on it.

2. PJ-223 is the best argument for §9 and §11 that exists, and it is on live data.
   Eisa Universe holds 2,096 .md on disk against 1,890 indexed rows — 798 of the missing
   ones in Constellation PKM, a registered library. The cold-start gate asks
   COUNT(*) WHERE library_name = ? , gets 1, and concludes "already indexed", so the
   recovery that exists for exactly this gap can never fire. Proven pre-existing from
   MIG-108's own pre-unification snapshot. §9 would report it; §11 would fix it. Consider
   quoting that number to the Boss when §9's notice ships.

Three standing rules this migration keeps proving:
1. Reproduce-First is absolute. Nothing ships against a defect not reproduced on demand.
2. "A builder is not a healer." Verify the plan against the code at every step, even
   though it passed an Architect phase and two adversarial reviews. §8 added a corollary:
   behaviour tests can pin a mechanism but not a wiring when every entry point takes an
   AppHandle and the crate has no Tauri test harness — §8 needed a source-level guard.
3. No test material reaches me except through tutorial-auditor → ui-inspector → me. The
   inspector's default verdict is REJECTED. It rejected §8's test twice, both times
   correctly. Pin the repo path (E:\مشاريع كلاود\Constellation) in every agent brief —
   an agent read a stale .claude/worktrees copy and wrongly reported §8 missing.

Testing notes: my federated universe is "Eisa Universe", not "كون عيسى". The registry
file is not a reliable indicator of which universe the app is running — verify from that
universe's own boot-perf.latest.json / diagnostics.log. An in-app save cannot exercise the
watcher path; only an edit made outside Constellation does.

Standing: I test and pass every build BEFORE commit — no exceptions.
Do not start MIG-109 or MIG-110; both are allocated and deliberately unscheduled.
