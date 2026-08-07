# Next-session prompt — paste this verbatim

```
Read docs/HANDOVER-2026-08-03-pj207.md first, then
docs/Constellation Orientation & Onboarding v3.83.md,
docs/Constellation Pending Jobs v1.67.md, and docs/PJ-207-Index-Repair-Plan.md.

Working on: PJ-207 §8 — the index stops adopting notes that belong to a linked universe.

Context: PJ-207 is a 15-step /migration. §1–§7 are shipped, pushed and Boss-tested
(head 53eaaa90). §8–§15 are unstarted and their plan is unchanged. The door itself —
the thing that finally makes the repair reachable — is §11.

§8 closes Charter W2-9 (OPEN, HIGH): the walk indexes foreign cUniverse notes into
the active universe's search.db. READ THE ARCHITECT DOC'S §8 CORRECTION BEFORE
PLANNING ANYTHING — scoping only the walk does NOT close it. reconcile.rs draws its
roots from the recursive set and re-adopts orphans, so removal without scoping BOTH
passes oscillates forever: delete → ledger append+fsync → re-adopt, every launch.
Both must route through universe::own_libraries_for_root, which already exists and
was written for exactly this write-scope discipline. Note its one trap: it reads
libraries.json with unwrap_or_default(), so an unreadable file yields an empty list —
safe for the boot pass, but for a repair it means "walk nothing and report success".

Three standing rules this migration keeps proving:
1. Reproduce-First is absolute. Nothing ships against a defect that has not been
   reproduced on demand.
2. "A builder is not a healer." The plan has been wrong three times in that exact
   shape — a back-fill computes and THEN stamps, so routing it through the healer's
   stamp gate makes it a permanent no-op. Verify the plan against the code at every
   step, even though it passed an Architect phase and two adversarial reviews.
3. No test material reaches me except through tutorial-auditor → ui-inspector →
   me. The inspector's default verdict is REJECTED. This is law in CLAUDE.md and it
   is not skippable for a small test.

Standing: I test and pass every build BEFORE commit — no exceptions.
Do not start MIG-109 or MIG-110; both are allocated and deliberately unscheduled.
```

---

## Why this file exists

Standing Order (session close) requires the complete PCS **plus** the handover **plus** a
ready-to-paste next-session prompt — *"if Eisa has to ask, it was incomplete."* On
2026-08-03 he had to ask. Written after the fact and committed so the next close has the
shape in front of it.
