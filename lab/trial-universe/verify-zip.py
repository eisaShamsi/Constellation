"""Quick validation of the release ZIP: integrity, entry count, arcname format."""
import zipfile
p = r"\\?\E:\Backups\Constellation\constellation-trial-universe-v1-20260414.zip"
with zipfile.ZipFile(p, "r") as z:
    bad = z.testzip()
    print("Integrity:", "OK" if bad is None else f"CORRUPT at {bad}")
    names = z.namelist()
    print(f"Entries: {len(names):,}")
    print("Sample arcnames:")
    for n in names[:5]:
        print("  ", n)
    bs_count = sum(1 for n in names if "\\" in n)
    print(f"Entries containing backslash: {bs_count}")
    # A well-formed cross-platform ZIP has forward slashes only.
