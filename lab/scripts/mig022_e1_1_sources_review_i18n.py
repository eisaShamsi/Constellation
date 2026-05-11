#!/usr/bin/env python3
"""
MIG-022 §E.1.1 — Backfill `sources.review.*` keys across 13 non-en/non-ar locales.

Boss-Test Gate 1 Stage 2 surfaced that the 7 sources.review.* keys
(axis.horizontal, axis.vertical, accept, edit, reject, save, cancel)
were present in en.json + ar.json but missing from all 13 backfilled
locales. Same gap shape as V3-§10.D.2 (settings.classifier) — V3-§10.D
swept cece.* keys and missed sources.review.*.

Inserts the missing keys into each locale's existing sources.review
sub-object (creating it if absent). Preserves order via Python 3.7+
dict semantics. UTF-8 ensure_ascii=False.

Idempotent: skips a locale that already has all 7 keys populated.
"""
import json
from pathlib import Path

I18N_DIR = Path(r"E:\مشاريع كلاود\Constellation\src\lib\i18n")

# Per-locale translations for the 7 sources.review.* keys.
# Keys mirror the en.json shape: axis.horizontal/vertical are nested,
# the other 5 are flat sibling keys.
TRANSLATIONS = {
    "de": {"axis_horizontal": "Quellen", "axis_vertical": "Inhaltstyp",
           "accept": "Annehmen", "edit": "Bearbeiten", "reject": "Ablehnen",
           "save": "Speichern", "cancel": "Abbrechen"},
    "es": {"axis_horizontal": "Fuentes", "axis_vertical": "Tipo de contenido",
           "accept": "Aceptar", "edit": "Editar", "reject": "Rechazar",
           "save": "Guardar", "cancel": "Cancelar"},
    "fa": {"axis_horizontal": "منابع", "axis_vertical": "نوع محتوا",
           "accept": "تأیید", "edit": "ویرایش", "reject": "رد",
           "save": "ذخیره", "cancel": "لغو"},
    "fr": {"axis_horizontal": "Sources", "axis_vertical": "Type de contenu",
           "accept": "Accepter", "edit": "Modifier", "reject": "Rejeter",
           "save": "Enregistrer", "cancel": "Annuler"},
    "he": {"axis_horizontal": "מקורות", "axis_vertical": "סוג תוכן",
           "accept": "אישור", "edit": "עריכה", "reject": "דחייה",
           "save": "שמירה", "cancel": "ביטול"},
    "hi": {"axis_horizontal": "स्रोत", "axis_vertical": "सामग्री प्रकार",
           "accept": "स्वीकारें", "edit": "संपादित करें", "reject": "अस्वीकार करें",
           "save": "सहेजें", "cancel": "रद्द करें"},
    "ja": {"axis_horizontal": "ソース", "axis_vertical": "コンテンツタイプ",
           "accept": "承認", "edit": "編集", "reject": "拒否",
           "save": "保存", "cancel": "キャンセル"},
    "ko": {"axis_horizontal": "출처", "axis_vertical": "콘텐츠 유형",
           "accept": "수락", "edit": "편집", "reject": "거부",
           "save": "저장", "cancel": "취소"},
    "pt": {"axis_horizontal": "Fontes", "axis_vertical": "Tipo de conteúdo",
           "accept": "Aceitar", "edit": "Editar", "reject": "Rejeitar",
           "save": "Salvar", "cancel": "Cancelar"},
    "ru": {"axis_horizontal": "Источники", "axis_vertical": "Тип содержимого",
           "accept": "Принять", "edit": "Редактировать", "reject": "Отклонить",
           "save": "Сохранить", "cancel": "Отменить"},
    "tr": {"axis_horizontal": "Kaynaklar", "axis_vertical": "İçerik türü",
           "accept": "Kabul et", "edit": "Düzenle", "reject": "Reddet",
           "save": "Kaydet", "cancel": "İptal"},
    "ur": {"axis_horizontal": "ذرائع", "axis_vertical": "مواد کی قسم",
           "accept": "قبول", "edit": "ترمیم", "reject": "رد",
           "save": "محفوظ کریں", "cancel": "منسوخ"},
    "zh": {"axis_horizontal": "来源", "axis_vertical": "内容类型",
           "accept": "接受", "edit": "编辑", "reject": "拒绝",
           "save": "保存", "cancel": "取消"},
}


def ensure_review_block(sources: dict, trans: dict) -> bool:
    """Ensure sources.review has the 7 expected keys; return True if file changed."""
    review = sources.setdefault("review", {})
    changed = False

    axis = review.setdefault("axis", {})
    if "horizontal" not in axis:
        axis["horizontal"] = trans["axis_horizontal"]
        changed = True
    if "vertical" not in axis:
        axis["vertical"] = trans["axis_vertical"]
        changed = True

    for key in ("accept", "edit", "reject", "save", "cancel"):
        if key not in review:
            review[key] = trans[key]
            changed = True

    return changed


def main():
    for locale, trans in TRANSLATIONS.items():
        path = I18N_DIR / f"{locale}.json"
        if not path.exists():
            print(f"SKIP {locale}: file not found at {path}")
            continue
        with path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        sources = data.setdefault("sources", {})
        if ensure_review_block(sources, trans):
            with path.open("w", encoding="utf-8") as f:
                json.dump(data, f, ensure_ascii=False, indent=2)
                f.write("\n")
            print(f"OK   {locale}: sources.review.* backfilled (7 keys)")
        else:
            print(f"SKIP {locale}: sources.review.* already complete")


if __name__ == "__main__":
    main()
