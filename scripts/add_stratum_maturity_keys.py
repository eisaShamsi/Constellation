"""Add stratum/maturity legend i18n keys to all 15 locales."""
import json
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "src" / "lib" / "i18n"

TRANSLATIONS = {
    "en": {"colorByStratum": "Stratum", "colorByStratumHint": "Color by knowledge stratum (1\u20138)", "colorByMaturity": "Maturity", "colorByMaturityHint": "Color by note maturity state"},
    "ar": {"colorByStratum": "الطبقة", "colorByStratumHint": "التلوين حسب طبقة المعرفة (1\u20138)", "colorByMaturity": "النضج", "colorByMaturityHint": "التلوين حسب حالة نضج الملاحظة"},
    "de": {"colorByStratum": "Schicht", "colorByStratumHint": "Nach Wissensschicht einf\u00e4rben (1\u20138)", "colorByMaturity": "Reife", "colorByMaturityHint": "Nach Notizreifegrad einf\u00e4rben"},
    "es": {"colorByStratum": "Estrato", "colorByStratumHint": "Colorear por estrato de conocimiento (1\u20138)", "colorByMaturity": "Madurez", "colorByMaturityHint": "Colorear por estado de madurez de la nota"},
    "fa": {"colorByStratum": "لایه", "colorByStratumHint": "رنگ\u200cآمیزی بر اساس لایه دانش (1\u20138)", "colorByMaturity": "بلوغ", "colorByMaturityHint": "رنگ\u200cآمیزی بر اساس حالت بلوغ یادداشت"},
    "fr": {"colorByStratum": "Strate", "colorByStratumHint": "Colorer par strate de connaissance (1\u20138)", "colorByMaturity": "Maturit\u00e9", "colorByMaturityHint": "Colorer par \u00e9tat de maturit\u00e9 de la note"},
    "he": {"colorByStratum": "שכבה", "colorByStratumHint": "צביעה לפי שכבת ידע (1\u20138)", "colorByMaturity": "בשלות", "colorByMaturityHint": "צביעה לפי מצב הבשלות של הפתק"},
    "hi": {"colorByStratum": "स्तर", "colorByStratumHint": "ज्ञान स्तर के अनुसार रंग (1\u20138)", "colorByMaturity": "परिपक्वता", "colorByMaturityHint": "नोट परिपक्वता अवस्था के अनुसार रंग"},
    "ja": {"colorByStratum": "\u5c64", "colorByStratumHint": "\u77e5\u8b58\u5c64\uff081\u20138\uff09\u3067\u8272\u5206\u3051", "colorByMaturity": "\u6210\u71df\u5ea6", "colorByMaturityHint": "\u30ce\u30fc\u30c8\u306e\u6210\u71df\u5ea6\u3067\u8272\u5206\u3051"},
    "ko": {"colorByStratum": "\uacc4\uce35", "colorByStratumHint": "\uc9c0\uc2dd \uacc4\uce35(1\u20138)\uc5d0 \ub530\ub77c \uc0c9\uc0c1 \uc9c0\uc815", "colorByMaturity": "\uc131\uc219\ub3c4", "colorByMaturityHint": "\ub178\ud2b8 \uc131\uc219\ub3c4 \uc0c1\ud0dc\uc5d0 \ub530\ub77c \uc0c9\uc0c1 \uc9c0\uc815"},
    "pt": {"colorByStratum": "Estrato", "colorByStratumHint": "Colorir por estrato de conhecimento (1\u20138)", "colorByMaturity": "Maturidade", "colorByMaturityHint": "Colorir por estado de maturidade da nota"},
    "ru": {"colorByStratum": "\u0421\u0442\u0440\u0430\u0442\u0430", "colorByStratumHint": "\u041e\u043a\u0440\u0430\u0441\u0438\u0442\u044c \u043f\u043e \u0441\u0442\u0440\u0430\u0442\u0435 \u0437\u043d\u0430\u043d\u0438\u0439 (1\u20138)", "colorByMaturity": "\u0417\u0440\u0435\u043b\u043e\u0441\u0442\u044c", "colorByMaturityHint": "\u041e\u043a\u0440\u0430\u0441\u0438\u0442\u044c \u043f\u043e \u0441\u043e\u0441\u0442\u043e\u044f\u043d\u0438\u044e \u0437\u0440\u0435\u043b\u043e\u0441\u0442\u0438 \u0437\u0430\u043c\u0435\u0442\u043a\u0438"},
    "tr": {"colorByStratum": "Katman", "colorByStratumHint": "Bilgi katman\u0131na g\u00f6re renklendir (1\u20138)", "colorByMaturity": "Olgunluk", "colorByMaturityHint": "Not olgunluk durumuna g\u00f6re renklendir"},
    "ur": {"colorByStratum": "تہ", "colorByStratumHint": "علم کی تہ کے مطابق رنگ کریں (1\u20138)", "colorByMaturity": "پختگی", "colorByMaturityHint": "نوٹ کی پختگی کے مطابق رنگ کریں"},
    "zh": {"colorByStratum": "\u5c42\u7ea7", "colorByStratumHint": "\u6309\u77e5\u8bc6\u5c42\u7ea7\u7740\u8272\uff081\u20138\uff09", "colorByMaturity": "\u6210\u719f\u5ea6", "colorByMaturityHint": "\u6309\u7b14\u8bb0\u6210\u719f\u5ea6\u7740\u8272"},
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
