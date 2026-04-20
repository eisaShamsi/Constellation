#!/usr/bin/env python3
"""Add panels.links tooltip key to all 15 locales."""
import json
from pathlib import Path

TRANSLATIONS = {
    "en": "Link Dashboard",
    "ar": "لوحة الروابط",
    "de": "Link-Dashboard",
    "es": "Panel de Enlaces",
    "fa": "داشبورد پیوندها",
    "fr": "Tableau des liens",
    "he": "לוח הקישורים",
    "hi": "लिंक डैशबोर्ड",
    "ja": "リンクダッシュボード",
    "ko": "링크 대시보드",
    "pt": "Painel de Ligações",
    "ru": "Панель ссылок",
    "tr": "Bağlantı Panosu",
    "ur": "لنک ڈیش بورڈ",
    "zh": "链接仪表盘",
}

root = Path(__file__).resolve().parents[2] / "src" / "lib" / "i18n"
for locale, label in TRANSLATIONS.items():
    p = root / f"{locale}.json"
    data = json.loads(p.read_text(encoding="utf-8"))
    data.setdefault("panels", {})["links"] = label
    p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{locale}: panels.links={label!r}")
