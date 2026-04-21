#!/usr/bin/env python3
"""Add archive-related keys to all 15 locales.

- linkConfidence.archive    (right-click menu item label)
- linkDashboard.archived    (tab label)
- linkDashboard.noArchived  (empty-state message)
- linkDashboard.unarchiveTitle (tooltip on restore button)
- linkDashboard.loading     (loading state)
"""
import json
from pathlib import Path

STRINGS = {
    # (archive-action, archived-tab, noArchived, unarchiveTitle, loading)
    "en": ("Archive link", "Archived", "No archived links.", "Restore this link", "Loading…"),
    "ar": ("أرشفة الرابط", "مؤرشف", "لا توجد روابط مؤرشفة.", "استعادة هذا الرابط", "جارٍ التحميل…"),
    "de": ("Link archivieren", "Archiviert", "Keine archivierten Links.", "Diesen Link wiederherstellen", "Lädt…"),
    "es": ("Archivar enlace", "Archivados", "No hay enlaces archivados.", "Restaurar este enlace", "Cargando…"),
    "fa": ("بایگانی پیوند", "بایگانی‌شده", "هیچ پیوند بایگانی‌شده‌ای نیست.", "بازیابی این پیوند", "در حال بارگذاری…"),
    "fr": ("Archiver le lien", "Archivés", "Aucun lien archivé.", "Restaurer ce lien", "Chargement…"),
    "he": ("העבר קישור לארכיון", "בארכיון", "אין קישורים בארכיון.", "שחזר קישור זה", "טוען…"),
    "hi": ("लिंक संग्रहित करें", "संग्रहित", "कोई संग्रहित लिंक नहीं।", "इस लिंक को पुनर्स्थापित करें", "लोड हो रहा है…"),
    "ja": ("リンクをアーカイブ", "アーカイブ済み", "アーカイブされたリンクはありません。", "このリンクを復元", "読み込み中…"),
    "ko": ("링크 보관", "보관됨", "보관된 링크가 없습니다.", "이 링크 복원", "로드 중…"),
    "pt": ("Arquivar link", "Arquivados", "Nenhum link arquivado.", "Restaurar este link", "Carregando…"),
    "ru": ("Архивировать ссылку", "В архиве", "Нет архивных ссылок.", "Восстановить эту ссылку", "Загрузка…"),
    "tr": ("Bağlantıyı arşivle", "Arşivlenmiş", "Arşivlenmiş bağlantı yok.", "Bu bağlantıyı geri yükle", "Yükleniyor…"),
    "ur": ("لنک محفوظ کریں", "محفوظ شدہ", "کوئی محفوظ شدہ لنک نہیں۔", "اس لنک کو بحال کریں", "لوڈ ہو رہا ہے…"),
    "zh": ("归档链接", "已归档", "没有已归档的链接。", "恢复此链接", "加载中…"),
}

root = Path(__file__).resolve().parents[2] / "src" / "lib" / "i18n"

for locale, (archive, archived, none, untitle, loading) in STRINGS.items():
    p = root / f"{locale}.json"
    data = json.loads(p.read_text(encoding="utf-8"))

    data.setdefault("linkConfidence", {})["archive"] = archive

    ld = data.setdefault("linkDashboard", {})
    ld["archived"] = archived
    ld["noArchived"] = none
    ld["unarchiveTitle"] = untitle
    ld["loading"] = loading

    p.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{locale}: archive+archived+noArchived+unarchiveTitle+loading written")

print("\nAll 15 locales updated.")
