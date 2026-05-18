---
id: peirce
name: Peirce
family: modern-western
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# Peirce

**ファミリー**: 近代西洋 · **形状**: 扇形(3楔形)

## ヒーロー・メタファー

ドームは、Peirceがすべての経験とすべての推論の基礎にあると論じた
**3つの現象学的カテゴリ**に分割されます:

- **第一性(Firstness)** — 質、感情、可能性。何か赤いものが存在する
  *前の*「赤であるとはどういうことか」。
- **第二性(Secondness)** — 行為、反応、生の事実。実際の衝突、世界の
  抵抗。
- **第三性(Thirdness)** — 媒介、法、習慣、記号。第一性と第二性を
  結びつけるパターン; これがあの後に起こることを説明する規則性。

3つのセクターはそれぞれ120°に位置し、基本軸から+π/6回転されて
(§δ.1-fix-1)、ドーム上部のStratumラベルを通る区切り線がないように
されています。

## スコープ

**この伝統を使うべき時。** このコンテンツが*どのカテゴリの経験*を
記述するかが問いであるとき。記号論的作業、感じられた質と行われた事実と
説明的法を区別すること、議論タイプの分析(あるPeirce的読みでは
アブダクティブな第一性、デダクティブな第二性、インダクティブな第三性)
に優れています。

**この伝統を使うべきでない時。** コンテンツに現象学的切り口がないとき —
それがすべてデータ、またはすべて行為、またはすべて法であるとき、カテゴ
リ語彙は有用な分類を提供しません。また要求の厳しい伝統です: カテゴリを
正しく適用するには努力が必要で、素朴な読みは第三性を第二性に崩壊させる
傾向があります。

## 適用範囲

- 記号論、記号理論、コミュニケーション研究。
- 経験の現象学。
- 議論タイプ分析(アブダクション / デダクション / インダクション)。

## 系譜

アメリカン・プラグマティズム。Charles Sanders Peirce(1839–1914)、
プラグマティシズムと近代形式論理学の創始者は、彼のキャリア全体を通じて
3つのカテゴリを明確にした; それらは早期(「On a New List of Categories」、
1867)に登場し、彼の記号論的および現象学的作業でより洗練されたものに
なります。カテゴリは*普遍的*: Peirceは、それらが一つの領域だけでなく、
すべての現象の還元不可能な語彙を構成すると論じました。

## 批評

カテゴリは訓練なしに適用するのが難しいことで知られ、Peirce自身も自分の
提示を何度も改訂しました。分析的側面からの批判は、3つ組分割が網羅的で
あるかを問います; 現象学的側面からの批判は、Peirceのカテゴリは生きら
れた経験を捉えるには形式的すぎると論じます。現在のConstellationレンダ
リングは、ノート単位のフロントマター抽出がまだ出荷されていないため、
すべてのノートをデフォルトで第一性にします — 出荷されると、ユーザーは
意図したカテゴリにノートをオプトインできます。

## 引用

**一次資料。** Charles S. Peirce, "On a New List of Categories" (1867),
in *Writings of Charles S. Peirce*, vol. 2, ed. Edward C. Moore et al.
(Bloomington: Indiana University Press, 1984).

**現代。** T. L. Short, *Peirce's Theory of Signs* (Cambridge:
Cambridge University Press, 2007); Robert Lane, *Peirce on Realism and
Idealism* (Cambridge: Cambridge University Press, 2018).

## ノート単位のフロントマター

`peirce_category: firstness | secondness | thirdness`。Rust側で現在
欠落 — すべてのノートは視覚的に埋まったベースラインとしてデフォルトで
第一性です。
