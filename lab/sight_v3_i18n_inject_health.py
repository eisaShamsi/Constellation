"""MIG-019 §2D i18n — universe-health card labels.
Inject into all 15 locales. ar gets Arabic; the other 13 get English placeholders.
"""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src/lib/i18n"

AR = {
    "universeHealth": "صحة الكون",
    "modularity": "النمطية",
    "dominance": "الهيمنة",
    "entropy": "الإنتروبيا",
    "connectivity": "الاتصال",
    "healthy": "صحي",
    "caution": "تحذير",
    "imbalanced": "غير متوازن",
    "notes": "ملاحظات",
    "edges": "حواف",
    "communities": "مجتمعات",
    "clickStarHint": "انقر على نجمة لمشاهدة تفاصيلها.",
}

EN = {
    "universeHealth": "Universe health",
    "modularity": "Modularity",
    "dominance": "Dominance",
    "entropy": "Entropy",
    "connectivity": "Connectivity",
    "healthy": "healthy",
    "caution": "caution",
    "imbalanced": "imbalanced",
    "notes": "notes",
    "edges": "edges",
    "communities": "communities",
    "clickStarHint": "Click a star to see its details.",
}

LOCALES = ["ar", "de", "en", "es", "fa", "fr", "he", "hi", "ja", "ko", "pt", "ru", "tr", "ur", "zh"]

for locale in LOCALES:
    path = LOCALES_DIR / f"{locale}.json"
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    values = AR if locale == "ar" else EN

    sv3 = data.get("sightV3", {})
    if not sv3:
        print(f"  [{locale}] WARN: sightV3 namespace missing; skipping")
        continue

    sp = sv3.setdefault("sidePanel", {})
    for k in ["universeHealth", "modularity", "dominance", "entropy", "connectivity",
              "healthy", "caution", "imbalanced", "notes", "edges", "communities", "clickStarHint"]:
        sp[k] = values[k]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  [{locale}] OK")

print("Done — 15 locales updated.")
