"""Restructure `notePane["stage.fleeting"]` flat keys into a nested
`notePane.stage.{fleeting|literature|permanent|synthesis}` object so
$t() (which splits on '.') can resolve them. Idempotent: if the
nested form is already present the script just confirms and skips.
Also ensures a `noteStage` placeholder key ("— Stage —") exists.
"""
import json, os

PLACEHOLDER = {
    'en': '— Stage —', 'ar': '— المرحلة —', 'de': '— Stufe —',
    'es': '— Etapa —', 'fa': '— مرحله —', 'fr': '— Étape —',
    'he': '— שלב —', 'hi': '— चरण —', 'ja': '— 段階 —',
    'ko': '— 단계 —', 'pt': '— Estágio —', 'ru': '— Этап —',
    'tr': '— Aşama —', 'ur': '— مرحلہ —', 'zh': '— 阶段 —',
}

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', 'src', 'lib', 'i18n'))
for fname in sorted(os.listdir(ROOT)):
    if not fname.endswith('.json'): continue
    lang = fname[:-5]
    path = os.path.join(ROOT, fname)
    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    np = data.get('notePane')
    if not isinstance(np, dict):
        print(f'{lang}: no notePane block, skip'); continue

    # Harvest any flat stage.* keys (possibly alongside a partial nested form)
    flat = {}
    for k in list(np.keys()):
        if k.startswith('stage.'):
            suffix = k.split('.', 1)[1]
            flat[suffix] = np.pop(k)

    nested = np.get('stage') if isinstance(np.get('stage'), dict) else {}
    # Merge: prefer existing nested values, fill from flat
    for k, v in flat.items():
        nested.setdefault(k, v)
    if nested:
        np['stage'] = nested
    np.setdefault('noteStage', PLACEHOLDER.get(lang, '— Stage —'))

    with open(path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write('\n')
    print(f'{lang}: stage nested ({len(nested)} items), noteStage placeholder set')
