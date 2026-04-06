# Constellation — PCS Protocol
## Push + Commit + Standing Order

---

## What PCS Is

PCS is Constellation's mandatory checkpoint protocol. It runs after every milestone, phase, or significant body of work. PCS is not optional — it is the safety net that guarantees continuity across sessions, devices, and context resets.

**PCS stands for:**
- **P** — Push (code to remote)
- **C** — Commit (with descriptive message)
- **S** — Standing Order (tag, backup, session log, documentation)

---

## Why PCS Exists

### 1. Session Continuity
Claude Code sessions have context limits. When a session ends — whether by completion, context exhaustion, or interruption — all in-memory knowledge is lost. PCS captures the state of the project in durable form (git history, session logs, documentation) so the next session can pick up exactly where this one left off.

### 2. Cross-Device Sync
Constellation is developed across multiple devices (Windows, macOS, iOS). `git push` ensures every device has the latest code. Without PCS, work done on one machine is invisible to another.

### 3. Disaster Recovery
The ZIP backup is a last-resort safety net. If git becomes corrupted, if a bad commit needs reverting, or if the repository needs to be restored to a known-good state, the ZIP archive provides a clean snapshot.

### 4. Knowledge Preservation
The session log captures what was done, why it was done, what files were changed, and what remains to be done. This is institutional memory — it prevents repeated work, preserves design decisions, and documents lessons learned.

### 5. Documentation Parity
User-facing changes must be documented immediately. If a feature is built but not documented, users can't find it. PCS forces documentation to be concurrent with development, not an afterthought.

### 6. Milestone Tracking
Git tags mark significant points in the project's evolution. They enable quick rollback to any milestone, comparison between phases, and a clear history of the project's growth.

---

## The PCS Sequence

PCS is executed in a strict order. Each step depends on the previous.

### Step 1: Verify Build

Before committing anything, both frontend and backend must compile cleanly:

```bash
# Frontend
npx vite build
# Must show: ✓ built in Xs + ✔ done

# Backend
cargo check
# Must show: Finished `dev` profile
```

**If the build fails, fix it before proceeding.** Never commit broken code.

### Step 2: Commit (C)

Stage and commit the changed files with a descriptive message.

**Rules:**
- **Stage specific files** — never use `git add -A` or `git add .` (risks including secrets, large binaries)
- **Write a clear commit message** — summarize the "why", not just the "what"
- **Include Co-Authored-By** — credits the AI collaborator
- **Use HEREDOC format** — ensures proper multiline formatting

```bash
# Stage specific files
git add src/lib/components/Feature.svelte src/routes/+layout.svelte src/lib/i18n/*.json

# Commit with descriptive message
git commit -m "$(cat <<'EOF'
Brief summary of what changed and why

Detailed explanation:
- What was built
- What was fixed
- What was changed

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
```

**Commit message guidelines:**
- First line: imperative verb + concise summary (under 72 chars)
- Blank line
- Body: bullet points explaining the changes
- Focus on WHY, not WHAT (the diff shows what)
- Examples:
  - Good: "Fix Arabic indexing: disable destructive stemming, conservative prefixes"
  - Bad: "Update libraries.rs"
  - Good: "Extract NoteEditor wrapper — one component, all 7 call sites"
  - Bad: "Refactor code"

### Step 3: Push (P)

Push to the remote repository immediately after committing:

```bash
git push origin main
```

**Why immediately?** Because:
- Other devices need the code
- If the session ends unexpectedly, the work is safe
- The remote is the source of truth for cross-session continuity

### Step 4: Standing Order (S)

The Standing Order has five sub-steps:

#### S.1: Milestone Tag

Create a descriptive tag for the milestone and push it:

```bash
git tag milestone/<descriptive-name> <commit-hash>
git push origin --tags
```

**Tag naming convention:** `milestone/<feature-or-phase-name>`

Examples:
- `milestone/post-notepane-audit`
- `milestone/dashboard-split-companion`
- `milestone/index-nlp-complete`
- `milestone/constellation-map-phase1`
- `milestone/wikilink-navigation`

#### S.2: ZIP Backup

Create a portable archive of the current state:

```bash
git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"
```

**Why ZIP?** Because:
- It's independent of git (works even if repo is corrupted)
- It's a point-in-time snapshot (immutable)
- It can be restored anywhere (no git required)

To restore from ZIP:
```bash
# Unzip and use directly
unzip Constellation-<name>-YYYYMMDD.zip

# Or restore to git
git checkout milestone/<name>
```

#### S.3: Session Log

Update the session log at `lab/reports/SESSION-LOG-YYYY-MM-DD.md`:

```markdown
---

## Phase: <Phase Name>

### Commits: `<first-commit>` → `<last-commit>`
**Tag:** `milestone/<tag-name>`

### What Was Built
- Feature 1 description
- Feature 2 description

### Architecture / Key Decisions
- Why we chose approach X over Y
- What principle rules were applied

### Files Created
- `path/to/new/file.rs` — description
- `path/to/new/component.svelte` — description

### Files Modified
- `path/to/modified/file.svelte` — what changed
- `src/lib/i18n/*.json` (15 files) — new keys for feature

### Test Results
- Feature X: tested and confirmed ✓
- Feature Y: tested and confirmed ✓

### Open Items
- What needs to be done next
- Known issues to address
- Future improvements planned
```

**Why the session log?** Because:
- The next session reads it to understand where to start
- It captures design decisions that aren't in the code
- It tracks what was tested and what wasn't
- It lists open items so nothing is forgotten

#### S.4: Help Files

Update the English help files with any user-facing changes:

- `docs/help.uConstellation.World/<feature>/<feature>.md` — detailed help article
- `docs/User Manual.md` — section in the main user manual

**What to include:**
- What the feature is
- Why it matters
- How to use it (step-by-step)
- Any keyboard shortcuts or settings

#### S.5: All 14 Translations

Update all translated User Manuals with equivalent content:

```
docs/help.ar/User Manual.md  — Arabic
docs/help.de/User Manual.md  — German
docs/help.es/User Manual.md  — Spanish
docs/help.fa/User Manual.md  — Persian/Farsi
docs/help.fr/User Manual.md  — French
docs/help.he/User Manual.md  — Hebrew
docs/help.hi/User Manual.md  — Hindi
docs/help.ja/User Manual.md  — Japanese
docs/help.ko/User Manual.md  — Korean
docs/help.pt/User Manual.md  — Portuguese
docs/help.ru/User Manual.md  — Russian
docs/help.tr/User Manual.md  — Turkish
docs/help.ur/User Manual.md  — Urdu
docs/help.zh/User Manual.md  — Chinese
```

**Why all 14?** Because Constellation is language-first by design. Every feature must be documented in every supported language. This is not optional — it's a design principle.

#### S.6: Commit the SO

After completing all five sub-steps, commit and push the session log and documentation:

```bash
git add lab/reports/SESSION-LOG-*.md "docs/User Manual.md" docs/help.*/  docs/help.uConstellation.World/
git commit -m "$(cat <<'EOF'
SO: <Phase name> — session log + docs (all 15 locales)

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
EOF
)"
git push origin main
```

---

## When to Execute PCS

PCS runs at these moments:

| Trigger | Example |
|---------|---------|
| **Phase completion** | "CE Layer 2: Constellation Map — Phase 1 complete" |
| **Milestone reached** | "All 4 broken NotePane instances fixed" |
| **Significant feature shipped** | "Wikilink navigation working" |
| **Before ending a session** | "Context is getting heavy, PCS before continuing" |
| **User explicitly requests it** | "PCS" |
| **Before starting a risky refactor** | "PCS first, then we restructure the SS" |
| **After a major bug fix** | "Callout RTL + SS edit interruption fixed" |

**Rule of thumb:** If you would be upset to lose the work, PCS it.

---

## PCS Checklist (Quick Reference)

```
□ npx vite build — clean
□ cargo check — clean
□ git add (specific files)
□ git commit -m "descriptive message"
□ git push origin main
□ git tag milestone/<name> && git push origin --tags
□ git archive → ZIP backup
□ Session log updated
□ Help files updated (English)
□ User Manual updated (English)
□ All 14 translations updated
□ SO committed and pushed
```

---

## Recovery Procedures

### Restore from Milestone Tag
```bash
git checkout milestone/<name>
```

### Restore from ZIP
```bash
unzip "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"
```

### View All Milestones
```bash
git tag -l "milestone/*"
```

### Compare Current State to a Milestone
```bash
git diff milestone/<name>..HEAD --stat
```

### Rollback to a Milestone (destructive)
```bash
git reset --hard milestone/<name>
git push origin main --force  # Only if absolutely necessary
```

---

## Summary

PCS is not bureaucracy — it is the **immune system** of the development process. It protects against context loss, device mismatch, documentation debt, and session discontinuity. Every minute spent on PCS saves hours of confusion and rework in future sessions.

**When in doubt, PCS.**
