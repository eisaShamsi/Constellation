"""Add two graphView.* i18n keys (legend toggle tooltips) to all 15 locales."""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src" / "lib" / "i18n"

TRANSLATIONS = {
    "en": {"showLegend": "Show legend", "hideLegend": "Hide legend"},
    "ar": {"showLegend": "إظهار مفتاح الألوان", "hideLegend": "إخفاء مفتاح الألوان"},
    "de": {"showLegend": "Legende anzeigen", "hideLegend": "Legende ausblenden"},
    "es": {"showLegend": "Mostrar leyenda", "hideLegend": "Ocultar leyenda"},
    "fa": {"showLegend": "نمایش راهنما", "hideLegend": "پنهان کردن راهنما"},
    "fr": {"showLegend": "Afficher la légende", "hideLegend": "Masquer la légende"},
    "he": {"showLegend": "הצג מקרא", "hideLegend": "הסתר מקרא"},
    "hi": {"showLegend": "लेजेंड दिखाएँ", "hideLegend": "लेजेंड छिपाएँ"},
    "ja": {"showLegend": "凡例を表示", "hideLegend": "凡例を非表示"},
    "ko": {"showLegend": "범례 표시", "hideLegend": "범례 숨기기"},
    "pt": {"showLegend": "Mostrar legenda", "hideLegend": "Ocultar legenda"},
    "ru": {"showLegend": "Показать легенду", "hideLegend": "Скрыть легенду"},
    "tr": {"showLegend": "Göstergeyi göster", "hideLegend": "Göstergeyi gizle"},
    "ur": {"showLegend": "لیجنڈ دکھائیں", "hideLegend": "لیجنڈ چھپائیں"},
    "zh": {"showLegend": "显示图例", "hideLegend": "隐藏图例"},
}

for lang, keys in TRANSLATIONS.items():
    fp = LOCALES_DIR / f"{lang}.json"
    with open(fp, "r", encoding="utf-8") as f:
        data = json.load(f)
    ns = data.get("graphView")
    if not isinstance(ns, dict):
        print(f"WARN: no 'graphView' namespace in {lang}.json, skipping")
        continue
    for k, v in keys.items():
        ns[k] = v
    with open(fp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"  {lang}.json: +{len(keys)} keys")

print("done")
