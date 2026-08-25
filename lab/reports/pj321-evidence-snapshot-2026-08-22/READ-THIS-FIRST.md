# ⚠ `universes.json` in this folder is NOT the app's registry. Do not cite it.

*Added 2026-08-25, after the file was traced.*

This bundle was committed (`bbb6ba9e` / `5ae1036d`, 2026-08-22) as durable evidence for **PJ-321**
— the entry claiming Constellation was not writing its universe registry. **That claim is false,
and this file is why it looked true.**

`universes.json` here is a copy of a **stale snapshot held inside the Claude Desktop MSIX
container**, not of the file Constellation reads and writes:

```
fsutil hardlink list "C:\Users\ealsh\AppData\Roaming\world.uconstellation.app\universes.json"
  \Users\ealsh\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\world.uconstellation.app\universes.json
```

Three objects that could have differed — the committed blob, the working tree, and the live
container file — all hash to
`c20f9694c5b3d21c9dce964700250c6c7e3f614f3115db0c6c9d04aa17946afd`.
The PJ-321 entry recorded that invariance **as its finding**. The invariance is real; what it
demonstrates is that a snapshot does not change.

**The app was behaving correctly the whole time.** `Eisa Universe`'s `boot-perf.latest.json` for
the 2026-08-24 boot logs one `set_active_universe` 16 ms after process start, and that report is
written to the *active* universe's `.constellation`. `set_active_universe` hard-fails
(`universe.rs:1026`) when the id is absent from the registry, and writes `active_id` +
`save_registry` at `:1225-1226`. So the real registry held an `Eisa Universe` entry and was
written that day.

## The two siblings ARE genuine

`eisa-universe-boot-perf.latest.json` and `eisa-universe-diagnostics.log` came from `E:\`, which
is not redirected. They are unaffected and remain valid evidence.

## Standing rule this produced

**Any Constellation file read under `%APPDATA%` from a Claude session must be
`fsutil hardlink list`-checked before its contents are treated as fact.** If the result names
`…\Packages\Claude_…\LocalCache\…`, the bytes are a snapshot of some earlier moment, not the
current file. Files under `E:\` are not redirected.

Five PJ-321 "corroborations" were gathered by re-reading these same 277 bytes through the same
redirected path. Their agreement was recorded as mounting evidence, when no repetition of that
read could ever have returned anything else. The discriminating command took one line and was
never run — see `lab/reports/SESSION-LOG-2026-08-24.md` §21.
