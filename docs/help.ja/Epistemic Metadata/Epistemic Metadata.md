# Epistemic Metadata

> **翻訳に関する注記:** このヘルプトピックは、`help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md` にある正規の英語版から AI によって生成された翻訳です。ネイティブスピーカーによるレビューは未完了です。修正はプロジェクトリポジトリ経由でご報告ください。

*(MIG-022 §A — ギャップ分析 §6.1 スキーマ拡張)*

このトピックでは、ノートのより豊かな認識的分類のために Constellation が新たに認識する、少数の **オプションのフロントマター・フィールド** について説明します。これらはギャップ分析 (`docs/epistemic-content-gap-analysis.md`) — Constellation 認識的コンテンツエンジン (CECE) が分類に使う 2 軸モデル「ソース × コンテンツタイプ」では、あなたが「どのように知ったか」について記録したい全てを捉えきれないという認識 — に応じて追加されました。

これらのフィールドは **すべてオプション** です。これらを持たない既存のノートはそのまま動作します。ノートが追加のシグナルから恩恵を受ける種類の知識である場合に、手で (または将来は構造化エディター経由で) 追加します。

---

## フィールド一覧

### `held_by` — *これは誰の立場か?*

ノートが記述する立場を保持する人物を示す短い文字列。デフォルトは `user` (あなた自身の立場) です。使用しうる他の値:
- 学者の名前: `held_by: "al-Shāfiʿī"`
- 学派 (madhhab): `held_by: "Ḥanafī"`
- 歴史的人物: `held_by: "Aristotle"`

あなた自身ではなく *他者の* 立場を記録するノートを書くとき、それを示すのが `held_by` フィールドです。これがないと、Constellation は暗黙のうちにそのノートの認識的状態をあなた自身のものと仮定します — これは真剣な学術作業ではしばしば誤りです。

### `domain` — *これはどの主題分野についてか?*

学問領域タグのリスト。自由形式の `tags` フィールド (フォークソノミー / ムード / プロジェクト) とは区別され、`domain` は検索とフィルタリングのための構造化された学問領域/トピックフィールドです。例:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

`content_type: "proposition"` と `source: "inference"` に分類されたノートは、論理定理 (domain: `[logic, mathematics]`) かもしれないし、法学的見解 (domain: `[fiqh, ʿibādāt]`) かもしれません — 同じ認識的形状ですが、検索文脈は大きく異なります。`domain` はそのどちらかを指定できるようにします。

### `function` — *このノートは何のためか?*

ノートの意図された用途を識別する単一の文字列。認識される値:

- `reference` — 必要時に読む (定義、引用、後で参照する事実)
- `seed` — 育てる (まだ発展中の初期段階のアイデア)
- `actionable` — これで何かをする (タスク、フォローアップ、行うべき決定)
- `shipped` — 完成品 (公開されたエッセイ、納品された分析、閉じたループ)

CECE のコンテンツタイプ軸 (どの種類の知識かを示す) とは区別されます — `function` はノートで何を *する* かを示します。

### `provenance_civilization` — *どの伝統の語彙が機能しているか?*

ノートの語彙の文明的な足跡を識別するオプションの文字列。伝統固有のコーパスに対する検索に有用。例:

- `provenance_civilization: "sunni-usuli"` — スンニ派 *uṣūl al-fiqh* の伝統 (al-Bukhārī、al-Ghazālī、al-Āmidī)
- `provenance_civilization: "analytic-western"` — フレーゲ以降の分析哲学
- `provenance_civilization: "nyaya"` — pramāṇa 認識論のインド Nyāya 学派
- `provenance_civilization: "buddhist-pramana"` — 仏教の認識論的伝統 (Dignāga、Dharmakīrti)

ほとんどのノートはこれを必要としません。例えば、スンニ派 *uṣūl* と分析的英米認識論の両方に依拠するノートがあるとき、主な足跡を記録しておくと、将来のあなたが適切な比較材料を取り出すのに役立ちます。

### `updated_at` — *あなたの立場が最後に変わったのはいつか?*

ノートの認識的内容が最後に意図的に改訂された ISO 日付。ファイルシステムの `modified` タイムスタンプ (タイポ修正でも保存のたびに変わる) とは区別されます。`updated_at` は実際に立場を考え直したときに *あなたが* 設定するタイムスタンプです。

```yaml
updated_at: 2026-05-09
```

§6.3 時間軸の残りの部分 (ノート状態履歴) が完成したときに有用です — それまでは「自分の見解を最後に改訂した時」を記録する単一スナップショットフィールドです。

### `ikhtilāf` — *構造化された学術的不一致*

新フィールドの中で最も複雑なもの。ある問題に関する学者または学派間の構造化された不一致 — *ikhtilāf* — を `{school, position}` ペアのリストとして記録します。Constellation はこの編集のためのカスタムプロパティパネルウィジェットを提供しています。YAML を直接編集することもできます。

例:

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

`ikhtilāf` を持つノートは、いかなる単一の認識的状態にもありません — それは複数のエージェント間の *構造化された不一致* を記録するものです。このフィールドがないと、Constellation はそのようなノートを、これらの立場のうち 1 つを保持しているかのように扱ってしまい、誤りとなります。

プロパティパネルは各行を 2 つの入力 (school + position) と削除ボタンを持つエディタカードとしてレンダリングし、下部に「学派を追加」ボタンを表示します。

### `warrant` と `warrant_notes` — *パースされるが (今のところ) 不活性*

2 つのフィールドはディスクにパースされ保存されますが、**現在のところ UI には現れません**:

- `warrant: "mutawātir"` — ノートの主張の根拠 (warrant) のグレードラベル。スンニ派 *uṣūl* の階層は *mutawātir / mashhūr / āḥād* を使用し、特にハディースでは *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ* を使います。他の伝統には独自のグレード語彙があります。
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — warrant グレードを支持する自由テキスト。

これらは **Constellation Warrant Research ワークストリーム** がその分類器を出荷したときに使用準備ができます (複数月の研究プロジェクト; ギャップ分析 §6.2 を参照)。それまでは手で入力でき、データは保持されますが、何も表示されません。将来の warrant 対応クエリとバッジは、これらの値を直接読みます。

---

## これらのフィールドが現れる場所

ノートのフロントマターに新フィールドを記入すると、他の YAML フィールドと同じように、**プロパティパネル** (右サイドバー) に表示されます — キーごとに 1 行、型に適したエディタとともに:

- `held_by`、`function`、`provenance_civilization`、`warrant`、`warrant_notes` → テキスト入力
- `domain` → タグリスト (入力 + Enter で追加、各タグの × で削除)
- `updated_at` → 日付選択
- `ikhtilāf` → `school` / `position` 行と追加/削除ボタンを持つカスタムウィジェット

---

## `supersedes` についてはどうか?

`supersedes` は、技術的には単一ノートのプロパティではなく *ノート間の関係* です。Constellation はこれを YAML スカラーではなく、**型付きリンク** として扱います:

```markdown
This note replaces my earlier analysis: [[old-note-id|supersedes]]
```

ウィキリンクの `|supersedes` サフィックスは、これが `supersedes` 種別の型付きリンクであることを Constellation に伝えます — 専用のピル色 (スレートブルーグレー) を持ち、他の型付きリンクと並んで Backlinks + Outgoing Links パネルに表示され、Living Link Architecture (重み、ライフサイクル、トラバーサル数) に参加します。

これにより、ノート間の関係を 1 か所 — 型付きリンクシステム — に保ち、型付きリンクとフロントマタースカラーの間で分割しないようにします。`contradicts:` についても同じことが当てはまります (MIG-022 以前の語彙では既に型付きリンクでした)。

---

## これは何で *ない* か

これらのフィールドは現在、CECE 分類によって **消費されません**。CECE はソース × コンテンツタイプのみで分類します。新しいメタデータフィールドは、人間主導の検索、将来の warrant 対応分類器、そして時間軸 (それが完成したとき) のために記録されます。

特に:
- `function: "actionable"` は Tasks パネルにタスクを自動作成 *しません*
- `held_by: "al-Shāfiʿī"` は CECE がノートを分類する方法を変更 *しません*
- `domain: [fiqh]` は、検索クエリにそれを含めるように書かない限り、検索結果をフィルタリング *しません*

これらのフィールドは **スキーマ** — 追加できる、認識された語彙です。将来の MIG はそれらを消費する機能を出荷します (warrant 分類器、時間的クエリ、ドメイン対応フィルタリングなど)。

---

## 完成した例

スンニ派学派の、「夜明けの断食義務が日の有効性に重要か」に関する立場を記録するノート:

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

The classical Mālikī position (one niyyah for the month) is described
by [[Ibn-Rushd-bidayah|derives-from]] in the bidāyat al-mujtahid passage
on niyyah. My current view: [[ramadan-niyyah-personal|supersedes]]
my earlier note that conflated the Mālikī position with the Shāfiʿī one.
```

7 つの新フィールドのうち 6 つが入力されています; `warrant_notes` は省略 (まだ記録すべき伝承の詳細がない); `supersedes` と `derives-from` は YAML スカラーではなく本文中の型付きリンクとして。

---

*MIG-022 §A — このスキーマ拡張は、この Constellation ビルドに含まれます。Warrant Research ワークストリーム (別の Concept Paper、複数月) は、`warrant` フィールドを消費する warrant 分類器を出荷します。時間軸 (MIG-023、別の Architect サイクル) は `updated_at` と、より広範なノート状態履歴を消費します。*
