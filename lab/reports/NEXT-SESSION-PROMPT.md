# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after the Knowledge-Cockpit three-lens set shipped (Butterfly · Ledger · Orrery). Copy everything in the box.

---

> Read `docs/Constellation Orientation & Onboarding v3.36.md` first (highest version — the cockpit three-lens set, the registry-base palette unification, and the RTL/Arabic-plural fixes are in its "What changed in v3.36" preamble), then the handover `lab/reports/HANDOVER-2026-07-11-cockpit-lenses-complete.md`. Then `git pull origin main` and skim `git log --oneline -8`.
>
> **State:** the second-screen **Knowledge Cockpit** is complete and Boss-validated — a read-only contextual view with **three note-graph lenses on one chassis**: **Butterfly** (composition — in/out + type mix), **Ledger** (magnitude — exact counts), **Orrery** (time — alive vs cold + the going-cold alarm). One palette shared with the whole app (Link Types registry base + a Style-Setter "Note graph → Relationship colours" override). Localized in all 15 languages including RTL. The Aster and Heartwood were cut. Latest binary rebuilt 2026-07-11.
>
> **Open threads (pick with the Boss):**
> 1. **Auto-restore-tabs-on-relaunch** (Boss-wanted 2026-07-09; a Settings toggle, default ON — open tabs are not persisted across restart today).
> 2. The **safety-sweep backlog** (standing G2–G8 items in `docs/Constellation-Safety-Audit-CHARTER.md`).
> 3. Optional Orrery polish if the Boss raises it: switching directly between two expanded wings is fiddly (the others shrink to slivers) — a small always-clickable per-wing label handle would fix it.
>
> **Standing rules that bite here:** the Art Director & Team own UX/UI **design AND coding** (run the multi-agent Art-Director workflow — specialists → AD spec → competing engineers → adversarial judges → lead merge; don't hand-iterate visual design solo — `feedback_art_director_team_owns_ui`). Every build: `safety-inspection` diff-scoped if it touches a write/index/lifecycle path (the cockpit is read-only display → exempt), `/simplify`, svelte-check 0/0, `npm run build` before `cargo build --release`, and verify the binary mtime is newer than source before any Boss test. Test tutorials = staged, one stage at a time.
>
> **Gotcha if you run an Art-Director build workflow:** the agent returns the `.svelte` file HTML-escaped (`&lt; &gt; &amp;`) in its JSON string field — `html.unescape` it (single level) before writing to disk. Also null-check `$state` narrowing in keyboard handlers.
