# -*- coding: utf-8 -*-
"""Was the historical 27-34 s a COLD-cache cost, and is it gone after the vacuum?

Method: for every recorded boot, compute the gap since the previous boot. A long gap means the
OS page cache has been evicted (cold); a short gap means warm. If cold boots were slow BEFORE
and a long-gap boot is fast AFTER, fragmentation was the cause and the standing rule fixed it.
"""
import io, json, datetime

P = r'E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\boot-perf.history.jsonl'
rows = []
for l in io.open(P, encoding='utf-8', errors='replace'):
    l = l.strip()
    if not l:
        continue
    try:
        d = json.loads(l)
    except Exception:
        continue
    t = d.get('cache_snapshot_core_invoke_start_unix_ms')
    if not t:
        continue
    ens = None
    for n, ms in (d.get('cache_snapshot_core_server_timings') or []):
        if n == 'ensure_db':
            ens = ms
    if ens is None:
        continue
    rows.append((t / 1000.0, ens))
rows.sort()
# de-duplicate identical timestamps (the file records some boots twice)
uniq = []
for t, e in rows:
    if not uniq or abs(t - uniq[-1][0]) > 1:
        uniq.append((t, e))
rows = uniq

VACUUM_DONE = 1785655676  # [defrag] done

def bucket(gap_s):
    if gap_s > 1800:
        return 'COLD (>30m idle)'
    if gap_s > 300:
        return 'cool (5-30m)'
    return 'warm (<5m)'

for label, sel in (('BEFORE the vacuum', lambda t: t < VACUUM_DONE),
                   ('AFTER the vacuum', lambda t: t >= VACUUM_DONE)):
    stats = {}
    prev = None
    for t, e in rows:
        gap = (t - prev) if prev else 1e9
        prev = t
        if not sel(t):
            continue
        stats.setdefault(bucket(gap), []).append(e / 1000.0)
    print('=== %s ===' % label)
    for k in ['COLD (>30m idle)', 'cool (5-30m)', 'warm (<5m)']:
        v = sorted(stats.get(k, []))
        if not v:
            print('   %-18s (no samples)' % k)
            continue
        print('   %-18s n=%-4d median %6.1fs   best %5.1fs   worst %6.1fs'
              % (k, len(v), v[len(v) // 2], v[0], v[-1]))
    print()
