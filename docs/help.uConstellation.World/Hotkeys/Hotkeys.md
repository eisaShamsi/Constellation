---
aliases:
  - Hotkeys
  - Keyboard shortcuts
  - Shortcuts
  - Rebind a key
  - Change a shortcut
  - Custom shortcuts
  - Key bindings
  - New tab shortcut
description: Give any Constellation command the keys you prefer — how the Hotkeys screen records a combination, which combinations it refuses and why, and how to undo a change.
---

# Hotkeys

Constellation ships with a set of keyboard shortcuts, but none of them are fixed. Every command in
the app can answer to whatever combination you find natural, and you set that in one place:
**Settings → Hotkeys**.

The reason this screen exists is simple. A shortcut is only useful if it is the one your hands
already know. Somebody arriving from another notes app has years of muscle memory in their fingers,
and asking them to relearn a dozen combinations is asking them to type slower for a month. The
shortcuts Constellation suggests are a starting point, not a house style.

---

## Changing a shortcut

Open **Settings → Hotkeys**. You will see every command the app has, each with its current keys
beside it.

1. **Find the command.** The **Filter commands** box at the top narrows the list as you type. It
   matches both the command's name and its current keys, so you can search for `Ctrl+G` just as
   easily as for "Sky View" — useful when you want to know what a combination is currently doing
   before you take it.
2. **Click the keys** shown beside the command. If the command has no shortcut yet it shows **Not
   set** — click that instead.
3. The row changes to **Press keys...**. Press the combination you want. It is recorded and saved
   the moment you press it; there is no separate confirm step.
4. **To cancel**, click anywhere else on the screen without pressing a combination. The command
   keeps whatever it had.

Your choices are saved with your settings and apply immediately — no restart.

---

## Undoing a change

Two buttons appear beside a command, and each only appears when it has something to do:

- **Reset** — shown once a command has been customised. It returns *that one command* to the
  shortcut Constellation ships with. Your other customisations are untouched.
- **Clear** — shown whenever a command has any shortcut at all. It leaves the command with no
  keyboard shortcut. The command still works; you reach it from the command palette or its button.

---

## Combinations Constellation will not accept

Some combinations are refused, and when one is, the row tells you why rather than silently doing
nothing. There are three reasons.

**A plain key with no modifier.** A binding on a bare `S` would fire on stray presses anywhere
outside a text box, which is not something a rebinding screen should let you do to yourself by
accident. Function keys are the exception — `F2`, `F8` and their siblings have no typing meaning, so
they may be bound on their own.

**`Escape`.** Escape always closes whatever is open, and that is not negotiable. If it could be
taken by a command, one rebinding could strand you inside a full-page view — including the Settings
screen you rebound it from — with no way out.

**Keys the note editor needs for itself.** `Ctrl+Z`, `Ctrl+Y`, `Ctrl+X`, `Ctrl+C`, `Ctrl+V`,
`Ctrl+A`, `Ctrl+F`, `Ctrl+L` and the `Ctrl` arrow keys belong to editing. A command bound to one of
them would win, and the editor's own behaviour would stop happening: you would press undo in a note,
nothing would happen, no error would appear, and the edit you wanted back would be gone. Constellation
refuses the binding instead.

**If another command already uses the combination**, Constellation tells you which one, rather than
letting two commands answer to the same keys and leaving you to work out why the wrong thing keeps
happening.

---

## On macOS

The same bindings are shown with the symbols the platform uses — `⌘` for Command and `⇧` for Shift.
A shortcut you set on one platform reads correctly on the other.

---

## New tab

**New tab** is a command like any other, and it arrives bound to `Ctrl+Shift+T`. It opens an empty
tab, the same as clicking the **+** button beside the tab strip — which for a long time was the only
way to reach it. If you would rather it were something else, rebind it here like anything else.

---

## Related

- **[Panels](../Panels/Panels.md)** — where the information panels live
- **[Notes Management](../Notes%20Management/Notes%20Management.md)** — tabs, opening and closing notes
