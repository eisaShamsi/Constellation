# -*- coding: utf-8 -*-
import sqlite3, os
db = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db"
con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
p = r"E:\Cognitive Knowledge\Humanities\libraries\History\Ancient History\Ancient history.md"
row = con.execute("SELECT modified, length(content_hash) FROM note_meta WHERE name='Ancient history'").fetchone()
print("note_meta.modified (cached):", row[0] if row else None)
print("file mtime now (s):         ", int(os.path.getmtime(p)))
print("file body has '::' form:    ", "[[supports::" in open(p, encoding="utf-8").read()[:4000] or "supports::" in open(p, encoding="utf-8").read())
