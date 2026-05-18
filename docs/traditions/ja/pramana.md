---
id: pramana
name: pramāṇa
family: indian-nyaya
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# pramāṇa

**ファミリー**: インド・ニヤーヤ · **形状**: 扇形(4象限)

## ヒーロー・メタファー

ドームは**4つの妥当な知の象限**に分割され、それぞれが一種の認識的根拠を
持つノートを収容します。知識は*どれだけ成熟しているか*(Aristotelian)
ではなく、*どのようにして知られるようになったか*によって分類されます:
直接知覚、証拠からの推論、既知の事例からの類推、信頼できる証言を通じて。
pramāṇasは**種類であり、レベルではない** — ノートをある象限から別の象限
へ移動することは、根拠の変化であり、Confidenceの変化ではありません。

各象限内では、Aristotelianの放射状Stratumエンコーディングが保持される
ため、ノートの理解の深さは根拠種別の中でも読みやすいままです。
§δ.2-fix-1(2026-05-17)以降、象限は元のNE/SE/SW/NWではなくE/S/W/Nに位置
し、垂直軸をStratumラベルとの衝突から開放します。

## スコープ

**この伝統を使うべき時。** 知識がどのように*根拠付けられているか*を
一目で見たいとき — 仕事のどの程度が直接観察に対して、推論された結論に
対して、比較に対して、権威に対して依拠しているかの比率。認識的自己監査
に有用: 証言への過度な依存はないか? 推論は値しない重みを担っていないか?

**この伝統を使うべきでない時。** ノート全体で根拠が変動しない場合 —
例えば、すべて体験的なUniverse(すべてpratyakṣa)や、すべて引用駆動の
Universe(すべてśabda)は、このレンズで有用な構造を表面化しません。
また、知識源のクリーンな分類を許容しないコンテンツ(創作、思索、フィク
ション)にも適しません。

## 適用範囲

- 研究プロジェクト全体の認識的バランスの自己監査。
- 一目で一次資料と二次資料を区別する。
- 知識の認知行為分析を教える。

## 系譜

古典的インド・ニヤーヤ — 認識を、それが生じる妥当な手段を列挙することで
分析した形式的インド認識論の学派。4-pramāṇaニヤーヤの正典が、
Constellationが採用しているバージョン(他のインド学派は数え方が異なる —
サーンキャは3つ、ミーマーンサーは6つを認識)。スートラ時代のインドから
中世の註釈書まで; 今日もB. K. Matilal、J. N. Mohanty などの著作を通じて
生きた伝統。

## 批評

4-pramāṇaニヤーヤ変種を選ぶこと自体が学術的立場 — ミーマーンサーの
6-pramāṇa観(*arthāpatti*要請と*anupalabdhi*非把握を追加)は、宗教系譜
ルール(オリエンテーションv2.09)によって明示的に除外されています、
ヴェーダ権威に基づくため; 仏教のPramāṇavāda伝統(Dignāga、Dharmakīrti)
も同様に除外されています。他のインド哲学系譜のユーザーは、
Constellationのレンダリングを還元的と感じるかもしれません。

## 引用

**一次資料。** *Nyāya-Sūtra* 1.1.3(4つのpramāṇasの列挙)。Gautama,
*The Nyāya Sūtras of Gautama*, trans. Satisa Chandra Vidyābhūṣana,
rev. ed. Nandalal Sinha (Delhi: Motilal Banarsidass, 1990) で入手可能。

**現代。** J. N. Mohanty, *Classical Indian Philosophy* (Lanham:
Rowman & Littlefield, 2000), 17–34; Bimal Krishna Matilal,
*Perception: An Essay on Classical Indian Theories of Knowledge*
(Oxford: Clarendon Press, 1986), ch. 1.

## ノート単位のフロントマター

`pramana_kind: pratyaksha | anumana | upamana | shabda`。Rust側の
`LayoutCacheRow`拡張がランディングすると、このフィールドがデフォルト配置
(現在すべてのノート → `pratyaksha`)を上書きします。哲学的デフォルトは
弁護可能: すべての知識は、反省的に再分類されるまで知覚として始まる。
