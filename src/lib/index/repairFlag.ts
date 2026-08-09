/**
 * PJ-207 §2 (2026-08-03) — the index-repair rollback levers.
 *
 * Same shape as `rtlFlag.ts` / `cockpitFlag.ts` / `fmPlusFlag.ts`: one const each,
 * defined BEFORE the code they gate, so every later step of the migration can be
 * switched off without reverting a commit.
 *
 * ## `REPAIR_DOOR_ENABLED`
 *
 * Gates every USER-REACHABLE route to the index repair:
 *   - §9  the post-paint drift notice ("N notes changed while Constellation was closed")
 *   - §11 the Settings → Index control, and the "Repair now" action on the health alert bar
 *   - §13 the offer to remove duplicated linked-universe copies
 *
 * False leaves every shipped GUARD in place — the single-flight runner, the windowed
 * recompute passes, the confidence-preserving re-index — and removes only the doors.
 * That asymmetry is deliberate: the guards fix defects that exist today (a repair
 * already runs on library-add), so they must not be rolled back with the feature.
 *
 * ## `FULL_REREAD_ENABLED`
 *
 * Gates ONLY the "Full re-read" action (§14) — the mode that re-reads every note
 * regardless of whether its file changed. It ships **off**, by Boss ruling
 * (2026-08-03): its duration has never been measured, and the confirmation dialog
 * must state a real number rather than a vague warning. Measured floor so far is
 * **49.0 s of pure file I/O** on the 7,824-note universe (298 MB), with parsing,
 * link diffing and FTS re-tokenisation all on top — see
 * `lab/reports/PJ-207-REPRODUCTION-2026-08-03.md` §5b.
 *
 * The flip to `true` is its own commit, after the §M1 measurement, together with the
 * dialog copy that quotes the number.
 *
 * **Scope note.** These gate the frontend, which is where the user-reachable routes
 * are. When §14 adds the Rust-side `FullReread` scope it must ALSO refuse the request
 * server-side while this is false — a UI-only gate hides a feature, it does not make
 * it unreachable, and PJ-207 exists precisely because a reachability claim went
 * unverified for months.
 */

/** Every user-reachable route to the index repair. See the module doc. */
export const REPAIR_DOOR_ENABLED = true;

/**
 * The "Full re-read" mode only.
 *
 * **ON since 2026-08-09** (Boss ruling), after §M1 measured it: on Eisa Universe's four
 * own libraries — 1,258 notes — the ordinary repair takes **0.1 s** and a full re-read
 * takes **24.7 s** (~51 notes/s, 40-216/s by note size). The ordinary repair only stats;
 * the re-read opens every file. The confirmation dialog quotes those numbers, which is
 * what the ruling required.
 *
 * **This flag hides the CONTROL. It does not make the mode unreachable.** The gate that
 * refuses the request is `index_repair::FULL_REREAD_ENABLED` on the Rust side; a devtools
 * `invoke` bypasses this constant entirely. Both must move together — turning the feature
 * off by hiding the button alone reproduces the exact shape PJ-207 exists because of.
 */
export const FULL_REREAD_ENABLED = true;
