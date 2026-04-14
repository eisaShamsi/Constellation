"""
Build the Trial Universe release ZIP from the output/ directory.
Handles Windows long paths (>260 chars) by prefixing \\?\ when reading.
"""
import os
import sys
import zipfile

SRC_REL = "Constellation Discovery"
DST = r"\\?\E:\Backups\Constellation\constellation-trial-universe-v1-20260414.zip"

here = os.path.dirname(os.path.abspath(__file__))
out_dir = os.path.join(here, "output")
src_abs = os.path.join(out_dir, SRC_REL)
src_long = "\\\\?\\" + src_abs

print(f"Source: {src_abs}")
print(f"Destination: {DST}")
print(f"Walking with long-path prefix...")

if not os.path.isdir(src_long):
    print("ERROR: source not found")
    sys.exit(1)

zf = zipfile.ZipFile(DST, "w", zipfile.ZIP_DEFLATED, compresslevel=6)
count = 0
skipped = 0
total_bytes = 0

for root, dirs, files in os.walk(src_long):
    for f in files:
        p = os.path.join(root, f)
        # Arcname: relative to src_long, then re-prefixed with "Constellation Discovery"
        rel = os.path.relpath(p, src_long)
        arcname = os.path.join(SRC_REL, rel)
        try:
            sz = os.path.getsize(p)
            zf.write(p, arcname)
            count += 1
            total_bytes += sz
            if count % 2000 == 0:
                print(f"  {count:,} files added ({total_bytes/1024/1024:.0f} MB source)", flush=True)
        except Exception as e:
            skipped += 1
            if skipped <= 10:
                print(f"  skip: {arcname[:120]} ({e})", flush=True)

zf.close()
zip_size = os.path.getsize(DST) / 1024 / 1024
print(f"\nDone.")
print(f"  Files added:   {count:,}")
print(f"  Files skipped: {skipped}")
print(f"  Source size:   {total_bytes/1024/1024:.1f} MB")
print(f"  ZIP size:      {zip_size:.1f} MB")
