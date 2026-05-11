#!/usr/bin/env python3
"""
MIG-022 §A.4.a — Backfill 8 i18n keys for §A.2 + §A.3 across 13 locales.

Keys:
  linkTypes.supersedes                  (1 — for §A.2 typed-link pill)
  propertyEditor.typeNestedObjectList   (1 — for §A.1 PropertyType label)
  propertyEditor.ikhtilafAddRow         (1 — §A.3 widget)
  propertyEditor.ikhtilafRemoveRow      (1 — §A.3 widget)
  propertyEditor.ikhtilafSchoolLabel    (1)
  propertyEditor.ikhtilafSchoolPlaceholder (1)
  propertyEditor.ikhtilafPositionLabel  (1)
  propertyEditor.ikhtilafPositionPlaceholder (1)

en + ar already populated by hand (see §A.4.a commit body).
This script handles the 13 other locales.
"""
import json
from pathlib import Path

I18N_DIR = Path(r"E:\مشاريع كلاود\Constellation\src\lib\i18n")

LINK_TYPE_SUPERSEDES = {
    "de": "ersetzt",
    "es": "reemplaza",
    "fa": "جایگزین می‌کند",
    "fr": "remplace",
    "he": "מחליף",
    "hi": "प्रतिस्थापित करता है",
    "ja": "置き換える",
    "ko": "대체함",
    "pt": "substitui",
    "ru": "заменяет",
    "tr": "yerine geçer",
    "ur": "تبدیل کرتا ہے",
    "zh": "取代",
}

PROPERTY_EDITOR_KEYS = {
    "de": {
        "typeNestedObjectList": "Mehrzeilige Liste (z. B. ikhtilāf)",
        "ikhtilafAddRow": "Schule hinzufügen",
        "ikhtilafRemoveRow": "Zeile entfernen",
        "ikhtilafSchoolLabel": "Schule",
        "ikhtilafSchoolPlaceholder": "Schule",
        "ikhtilafPositionLabel": "Position",
        "ikhtilafPositionPlaceholder": "Position",
    },
    "es": {
        "typeNestedObjectList": "Lista de múltiples filas (p. ej. ikhtilāf)",
        "ikhtilafAddRow": "Añadir escuela",
        "ikhtilafRemoveRow": "Eliminar fila",
        "ikhtilafSchoolLabel": "Escuela",
        "ikhtilafSchoolPlaceholder": "Escuela",
        "ikhtilafPositionLabel": "Posición",
        "ikhtilafPositionPlaceholder": "Posición",
    },
    "fa": {
        "typeNestedObjectList": "فهرست چندردیفی (مثل ikhtilāf)",
        "ikhtilafAddRow": "افزودن مذهب",
        "ikhtilafRemoveRow": "حذف ردیف",
        "ikhtilafSchoolLabel": "مذهب",
        "ikhtilafSchoolPlaceholder": "مذهب",
        "ikhtilafPositionLabel": "موضع",
        "ikhtilafPositionPlaceholder": "موضع",
    },
    "fr": {
        "typeNestedObjectList": "Liste à plusieurs lignes (par ex. ikhtilāf)",
        "ikhtilafAddRow": "Ajouter une école",
        "ikhtilafRemoveRow": "Supprimer la ligne",
        "ikhtilafSchoolLabel": "École",
        "ikhtilafSchoolPlaceholder": "École",
        "ikhtilafPositionLabel": "Position",
        "ikhtilafPositionPlaceholder": "Position",
    },
    "he": {
        "typeNestedObjectList": "רשימה מרובת שורות (למשל ikhtilāf)",
        "ikhtilafAddRow": "הוסף בית מדרש",
        "ikhtilafRemoveRow": "הסר שורה",
        "ikhtilafSchoolLabel": "בית מדרש",
        "ikhtilafSchoolPlaceholder": "בית מדרש",
        "ikhtilafPositionLabel": "עמדה",
        "ikhtilafPositionPlaceholder": "עמדה",
    },
    "hi": {
        "typeNestedObjectList": "बहु-पंक्ति सूची (जैसे ikhtilāf)",
        "ikhtilafAddRow": "विद्यालय जोड़ें",
        "ikhtilafRemoveRow": "पंक्ति हटाएँ",
        "ikhtilafSchoolLabel": "विद्यालय",
        "ikhtilafSchoolPlaceholder": "विद्यालय",
        "ikhtilafPositionLabel": "स्थिति",
        "ikhtilafPositionPlaceholder": "स्थिति",
    },
    "ja": {
        "typeNestedObjectList": "複数行リスト (例: ikhtilāf)",
        "ikhtilafAddRow": "学派を追加",
        "ikhtilafRemoveRow": "行を削除",
        "ikhtilafSchoolLabel": "学派",
        "ikhtilafSchoolPlaceholder": "学派",
        "ikhtilafPositionLabel": "立場",
        "ikhtilafPositionPlaceholder": "立場",
    },
    "ko": {
        "typeNestedObjectList": "다중 행 목록 (예: ikhtilāf)",
        "ikhtilafAddRow": "학파 추가",
        "ikhtilafRemoveRow": "행 삭제",
        "ikhtilafSchoolLabel": "학파",
        "ikhtilafSchoolPlaceholder": "학파",
        "ikhtilafPositionLabel": "입장",
        "ikhtilafPositionPlaceholder": "입장",
    },
    "pt": {
        "typeNestedObjectList": "Lista de múltiplas linhas (ex. ikhtilāf)",
        "ikhtilafAddRow": "Adicionar escola",
        "ikhtilafRemoveRow": "Remover linha",
        "ikhtilafSchoolLabel": "Escola",
        "ikhtilafSchoolPlaceholder": "Escola",
        "ikhtilafPositionLabel": "Posição",
        "ikhtilafPositionPlaceholder": "Posição",
    },
    "ru": {
        "typeNestedObjectList": "Многострочный список (напр. ikhtilāf)",
        "ikhtilafAddRow": "Добавить школу",
        "ikhtilafRemoveRow": "Удалить строку",
        "ikhtilafSchoolLabel": "Школа",
        "ikhtilafSchoolPlaceholder": "Школа",
        "ikhtilafPositionLabel": "Позиция",
        "ikhtilafPositionPlaceholder": "Позиция",
    },
    "tr": {
        "typeNestedObjectList": "Çok satırlı liste (ör. ikhtilāf)",
        "ikhtilafAddRow": "Mezhep ekle",
        "ikhtilafRemoveRow": "Satırı kaldır",
        "ikhtilafSchoolLabel": "Mezhep",
        "ikhtilafSchoolPlaceholder": "Mezhep",
        "ikhtilafPositionLabel": "Pozisyon",
        "ikhtilafPositionPlaceholder": "Pozisyon",
    },
    "ur": {
        "typeNestedObjectList": "کثیر سطری فہرست (مثلاً ikhtilāf)",
        "ikhtilafAddRow": "مذہب شامل کریں",
        "ikhtilafRemoveRow": "صف ہٹائیں",
        "ikhtilafSchoolLabel": "مذہب",
        "ikhtilafSchoolPlaceholder": "مذہب",
        "ikhtilafPositionLabel": "موقف",
        "ikhtilafPositionPlaceholder": "موقف",
    },
    "zh": {
        "typeNestedObjectList": "多行列表（如 ikhtilāf）",
        "ikhtilafAddRow": "添加学派",
        "ikhtilafRemoveRow": "删除行",
        "ikhtilafSchoolLabel": "学派",
        "ikhtilafSchoolPlaceholder": "学派",
        "ikhtilafPositionLabel": "立场",
        "ikhtilafPositionPlaceholder": "立场",
    },
}


def main():
    for locale, supersedes_label in LINK_TYPE_SUPERSEDES.items():
        path = I18N_DIR / f"{locale}.json"
        if not path.exists():
            print(f"SKIP {locale}: file not found")
            continue
        with path.open("r", encoding="utf-8") as f:
            data = json.load(f)

        # 1. linkTypes.supersedes
        link_types = data.setdefault("linkTypes", {})
        if "supersedes" not in link_types:
            link_types["supersedes"] = supersedes_label
            added_lt = 1
        else:
            added_lt = 0

        # 2. propertyEditor.* (7 keys)
        pe = data.setdefault("propertyEditor", {})
        added_pe = 0
        for key, val in PROPERTY_EDITOR_KEYS[locale].items():
            if key not in pe:
                pe[key] = val
                added_pe += 1

        with path.open("w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
            f.write("\n")
        print(f"OK   {locale}: linkTypes.supersedes (+{added_lt}), propertyEditor.* (+{added_pe})")


if __name__ == "__main__":
    main()
