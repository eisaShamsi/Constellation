# Review Pulse

*(The Reviewer — your knowledge's call-back list)*

Review Pulse is where Constellation tells you **which notes need your attention right now, and why** — and prescribes the one healthy thing to do about each. Every other panel answers a question you have to go and ask ("how is my graph? what does this note connect to?"). The Reviewer is the only one that **comes to you**: it surfaces the notes that have decayed, drifted, or come adrift, ranked by urgency, each carrying a plain-language reason.

Think of it as a doctor's **call-back list**. It doesn't just say "something's wrong" — it diagnoses the condition, prescribes the cure, tells you how urgent it is and *why*, and then hands you off to the deeper instruments (the editor, the 360° Inspector, the Cataloger) to act.

> **Two surfaces, one idea.** The full **Reviewer** (the 🕐 clock icon in the left dock) is the *whole-library* queue. The **Review tab** in the right sidebar (the same 🕐 icon) shows just *the note you have open* — its own status. They share the same engine, so a note shows the same priority in both.

---

## Opening the Reviewer

Click the **🕐 clock** icon in the far-left dock. The Reviewer fills the window in two columns:

- **Left — the queue**, grouped into six lenses (below). Every lens is always listed; an empty one is greyed with a **0**. Click a lens header to collapse or expand it.
- **Right — the detail pane** for the note you've selected: what's wrong, the cure, how urgent, and the actions.

Click any note in the queue to load its detail on the right. Click a note's **name** (or **Connect**) to open it in the editor — a **‹ Reviewer** button then appears in the top tab strip to bring you straight back to where you were.

---

## The six lenses

A note can appear in more than one lens at once — each lens answers a different question, and they are never blended into a single score.

| Lens | What it means |
|---|---|
| 🥀 **Stale** | A note this one *leans on* (a load-bearing link — supports, contradicts, derives-from, part-of, supersedes) changed **after** you last reviewed this note. Your note may no longer reconcile with it. |
| 🔄 **Due for Review** | The review interval has elapsed — time to re-read and confirm it still holds. |
| 🧠 **Mental-Model Checkpoints** | A note you flagged as an assumption or model. *Do you still hold this view?* |
| 🔗 **Orphan — connect me** | A note with real content that **nothing links to yet**. It's outside your web of thinking. An orphan is an **alarm**, not clutter: connect it, or mark it a deliberate standalone. |
| ⚠ **Fragile — shore me up** | Many notes lean on this one, but it rests on little support. A single point of failure — give it firmer ground. |
| 📝 **Never reviewed** | A note that's been in your library a while but you've never given it a first read-through. |

---

## The detail pane: diagnosis → prescription

When you select a note, the right column reads top to bottom:

1. **Title + summary.** A one-or-two-sentence summary of the note always shows here (whatever your summary settings) so you know *what* you're being asked to revisit.
2. **The diagnosis** — the plain-language "why now," e.g. *"derives-from 'Evidence' changed on 2026-06-12."*
3. **The prescription** — the one healthy thing to do, e.g. *"Review it against 'Evidence' — reconcile your stance or update it."* For an orphan: *"Connect it to a related note — or mark it deliberately standalone."*
4. **Priority** — a number from 0 to 100, shown as a **recipe** (see below).
5. **Facts** — its maturity (seed / sapling / evergreen / canonical / wilting), its connections ("12 in · 4 out"), and when you last reviewed it.
6. **Actions** + **hand-offs** (below).

---

## Priority you can read — and overrule

The priority number isn't a black box. It's computed from the note's situation and shown as a **bar split into its reasons**, each labelled and adding up exactly to the number — for example:

> **63**  ·  *Time pressure +31 · Depended on +14 · Maturity +10 · …*

It combines two things, the classic urgent-vs-important split:

- **Urgency** — how overdue or stale it is, and how disruptive the change was (a *contradiction* landing under you is more urgent than a supporting note merely shifting).
- **Importance** — how many notes depend on it, how mature it is, and whether it's fragile.

You're always in charge of the number. **Drag the slider** to set your own priority; it's then badged **"manual,"** shows you what the computed value *would* be, and offers **"Reset to computed"** to hand control back to the engine. Your override sticks until you reset it.

---

## Acting on a note

Each lens offers the right verbs:

- **✓ Reviewed** — you've re-read it; it's confirmed and re-scheduled on the 1·3·7·14·30-day ladder. (This is the *only* action that advances "last reviewed" — merely opening a note does not count as reviewing it.)
- **🔗 Connect** (orphans) — opens the note so you can add a link.
- **👁 Snooze 7d** — hide a time-due note for a week. (Snooze applies only to the time-based lenses; a stale or fragile note isn't something a week's wait fixes.)
- **🗄️ Dismiss** — stop tracking this note for review. For an orphan it reads **"Mark standalone"** — *this note is meant to stand alone, it isn't an orphan.*

**Hand-offs** at the bottom take you to the deeper instruments without leaving your train of thought: **Open in editor**, **Full context (360°)** (the note's full structural picture), and **Classify** (the Cataloger). The Reviewer triages; these explain.

---

## The note's own Review tab

Open any note, then the **🕐 Review** tab in the right sidebar. It shows *that* note's status — due / stale / a checkpoint / never-reviewed — with the same priority slider (computed by default, overridable, with reset) and the same ✓ / Snooze / Dismiss actions. It's the per-note view of everything above.

---

## Settings — the staleness grace period

By default a note is flagged **Stale** the day after a dependency changes. If that's too eager, go to **Settings → Review** and raise the **grace period** (in days, minimum 1): a dependency change then only flags the note once that many days have passed since your last review. Keep it patient if you make many small edits.
