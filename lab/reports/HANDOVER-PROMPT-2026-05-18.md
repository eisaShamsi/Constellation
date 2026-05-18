# Handover Prompt — 2026-05-18

Copy-paste the block below into a fresh Claude session to bootstrap it
on the Constellation project at this state.

---

You're a fresh Claude session picking up on **Constellation**, a Tauri v2 desktop app (Rust + SvelteKit/Svelte 5) — a Personal Knowledge Formulation system.

**Working directory**: `E:\مشاريع كلاود\Constellation` on branch `main`. Operate via absolute paths.

**State at handover**: MIG-026 baseline foundation 100% complete (just closed). 24 of 24 curated scholarly traditions shipped + 9 of 9 shape renderers implemented + 5 fix-iterations, all Boss-tested PASS, across 2026-05-17 evening through 2026-05-18 early morning. The Sight subsystem now lets the user view their full universe through any of 24 scholarly traditions (Aristotelian, pramāṇa, masādir, Polanyi, Peirce, Habermas, Dewey, Husserl, Longino, Mohist sān biǎo, Ibn Rushd burhān, Shāṭibī maqāṣid, Ibn Khaldūn ʿumrān, PaRDeS, Maimonidean prophecy, Talmudic 13 middot, Mencian sprouts, Wang Yangming, Korean Sŏngnihak, Mignolo pluriversal, Dussel transmodernity, Maldonado-Torres, Akan Wiredu, Ibuanyidanda). Plus MIG-027 (Sight follows the interface theme, shipped 2026-05-17) is verified across all 6 built-in themes.

**Remaining work**: MIG-026 Phases ι (manifests + ⓘ disclosure layer), κ (user-definable plugin loader), λ (15-locale translations), μ (ship gate + 3-agent audit). Plus a project-wide Pending Jobs pool (51 entries in v1.11 + 6 new MIG-026-derived ones to file at v1.12). Phase ι is the natural next step; PJ pool items can interleave.

---

## Required reads (in this order, before any other action)

**1. The last 30 versions of the project orientation** — read each version's `**What changed in vX.Y**` preamble at the top of the file for trajectory (what shipped when, what was decided, why direction shifted). Read the FULL BODY only from `v2.13.md` (the current canonical state). This is a deliberate deviation from the CLAUDE.md SO #6 rule that says "read only highest version"; Boss directive 2026-05-18 — the trail carries context that the v2.13 body alone does not preserve.

```
docs/Constellation Orientation & Onboarding v1.84.md
docs/Constellation Orientation & Onboarding v1.85.md
docs/Constellation Orientation & Onboarding v1.86.md
docs/Constellation Orientation & Onboarding v1.87.md
docs/Constellation Orientation & Onboarding v1.88.md
docs/Constellation Orientation & Onboarding v1.89.md
docs/Constellation Orientation & Onboarding v1.90.md
docs/Constellation Orientation & Onboarding v1.91.md
docs/Constellation Orientation & Onboarding v1.92.md
docs/Constellation Orientation & Onboarding v1.93.md
docs/Constellation Orientation & Onboarding v1.94.md
docs/Constellation Orientation & Onboarding v1.95.md
docs/Constellation Orientation & Onboarding v1.96.md
docs/Constellation Orientation & Onboarding v1.97.md
docs/Constellation Orientation & Onboarding v1.98.md
docs/Constellation Orientation & Onboarding v1.99.md
docs/Constellation Orientation & Onboarding v2.00.md
docs/Constellation Orientation & Onboarding v2.01.md
docs/Constellation Orientation & Onboarding v2.02.md
docs/Constellation Orientation & Onboarding v2.03.md
docs/Constellation Orientation & Onboarding v2.04.md
docs/Constellation Orientation & Onboarding v2.05.md
docs/Constellation Orientation & Onboarding v2.06.md
docs/Constellation Orientation & Onboarding v2.07.md
docs/Constellation Orientation & Onboarding v2.08.md
docs/Constellation Orientation & Onboarding v2.09.md
docs/Constellation Orientation & Onboarding v2.10.md
docs/Constellation Orientation & Onboarding v2.11.md
docs/Constellation Orientation & Onboarding v2.12.md
docs/Constellation Orientation & Onboarding v2.13.md
```

**2. `lab/reports/CONSTELLATION-SUBSYSTEM-STATE-2026-05-18.md`** — per-subsystem current state for all 19 core subsystems (Sight, Editor, Sky View, Constellation Map, 360.3D, Search Hub, Index Panel, Sources/CECE, Cognitive Engine, Theme System, Settings, Universe/Library/cUniverse, Arabic Engine + Lexical Bridge, Filename/Identity, Help, Boot Perf, Bases/Dataview/Importers, AI/Embeddings, Second Screen). Quick subsystem-state matrix at the bottom for at-a-glance status.

**3. `lab/reports/MIG-026-HANDOVER-2026-05-18.md`** — MIG-026 cascade specifics + remaining ι/κ/λ/μ phase specs + the 6 new MIG-026-derived PJs (store.ts TraditionId dedup, Concept Paper §4.1.2/§4.1.3 NE→E doc-drift, §8 Migrations table backfill, per-tradition frontmatter integration, CNS theming).

**4. `docs/Constellation Pending Jobs v1.11.md`** — canonical PJ list. 51 jobs across 9 sections. Per CLAUDE.md SO #8, cross-check any PJ against orientation body (NOT just preamble) and session logs before starting work on it — stale PJs are real.

**5. `lab/reports/SESSION-LOG-2026-05-18.md`** + `lab/reports/SESSION-LOG-2026-05-17.md`** — operational context. Yesterday's log carries the full γ → ζ.3 cascade; today's log carries θ + 2 fixes + the SHIP gate.

**6. `CLAUDE.md`** — standing rules. Read all of it. Key items: BASIC RULE (don't fabricate — say "I don't know"); Working Agreement #2 (one location, `E:\مشاريع كلاود\Constellation` main); Working Agreement #4 (validate against full architecture before shipping; spawn parallel agents for risk review); Working Agreement #5 (cross-check non-trivial fixes against proven methods via WebSearch); top principals (State the Function in Hand, Predecessor Lookup Rule, Stop-On-Correction Rule, Plan Approval = Build Approval, Testing Instructions Rule); Standing Orders (#1 session log per phase, #2 help+UM updates, #5 state-of-standing before pivot, #6 orientation v-bump same-commit, #7 MoCh every ~3h, #8 cross-check PJs); Performance Rule 8 (Write-Time Derivation).

---

## After the reads — verify state

```bash
git -C "E:/مشاريع كلاود/Constellation" log --oneline -5
```

Expect HEAD = `48a60550` (or later) — the session-open read directive commit. If older, you're not on the latest; do `git pull origin main`.

```bash
git -C "E:/مشاريع كلاود/Constellation" status
```

Expect clean tree on `main`.

```bash
ls "E:/مشاريع كلاود/Constellation/src-tauri/target/release/constellation.exe"
```

If the binary mtime is older than 2026-05-18 ~05:51, the last shipped state isn't built locally. A rebuild can wait until you need to Boss-test something.

---

## Before any code — state the function in hand

Per CLAUDE.md top principal "State the Function in Hand", write a one-line statement of what you're working on, named exactly as it appears in the orientation. Example formats:

> Working on: **MIG-026 Phase ι.1 — drafting the 24 tradition manifests at `docs/traditions/<id>.md`** per Plan §11.

> Working on: **PJ — `store.ts:3483` TraditionId literal-union duplicate cleanup** per the MIG-026 handover §2 PJ pool.

> Working on: **CNS theme inheritance audit** (likely MIG-028) per the MIG-026 handover §2 PJ pool.

If the function in hand isn't clear from Boss's first message, ASK before writing code.

---

## Then ask Boss which direction

The handover deliberately does NOT pick the next direction. Surface the options:

1. **MIG-026 Phase ι** — 24 tradition manifests + ⓘ disclosure-layer UI. Natural next step per Plan. ~3 days estimated.
2. **MIG-026 PJ pool** — quick wins from the 6 MIG-026-derived PJs (store.ts dedup, Concept Paper doc-drift, §8 Migrations backfill, per-tradition frontmatter integration, CNS theming).
3. **Project-wide PJ from v1.11** — any of the 51 entries. Cross-check per SO #8 before tackling.
4. **MIG-026 audit (3 parallel agents)** — could run early before Phases κ/λ/μ to catch any drift accumulated during the cascade. Insurance.
5. **Other** — Boss surfaces a different priority.

---

## Standing reminders

- **BASIC RULE**: if you don't know, say "I don't know". Don't fabricate file paths, line numbers, function names, badge taxonomies, prior-art summaries. Verify against the repo.
- **Stop-On-Correction**: if Boss says "wrong target", "no", "you're confused", or equivalent — STOP all in-flight edits, summarize what's changed since last approval, state corrected understanding, wait for explicit "proceed".
- **Plan Approval = Build Approval**: once Boss approves a plan, cascade autonomously through the build steps. Stops only at user-testable verification clauses, genuine architectural surprises, or plan completion.
- **Testing Instructions Rule**: every Boss test is a tutorial. Define the feature first, walk through every interaction in plain language. Pre-state, action, post-state. Failure modes spelled out.
- **Predecessor Lookup Rule**: before removing/relocating any feature, write a Predecessor → Replacement entry into the session log. Default: replacement lives in the same place as predecessor.
- **PJ cross-check**: never tackle a PJ without verifying against orientation body + session log first. Stale PJs are real (see CLAUDE.md SO #8 + the canonical violation note from 2026-05-06).
- **Address the user as "Eisa"**. The project docs call him "Boss" or "the user" — those stay unchanged in writing — but in chat, address him as Eisa.

---

End of handover prompt.

