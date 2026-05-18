---
id: masadir
name: masādir
family: sunni-islamic-usul
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# masādir

**家族**:逊尼派伊斯兰 *uṣūl* · **形状**:扇形(4 象限 + 4 扩展芯片)

## 核心隐喻

穹顶分为逊尼派 *uṣūl al-fiqh* 中**四种权威证明来源**:古兰经、圣行、
ijmāʿ(学者共识)和 qiyās(类比推理)。每一个都是不同*种类*的证明 —
而非同一种证明的不同程度 — 所以布局是扇形(范畴切片)而非同心圆
(分级深度)。穹顶下方,四个补充来源作为芯片就位:*istiḥsān*(法学偏
好)、*istiṣḥāb*(连续性推定)、*maṣlaḥa mursalah*(不受限制的公共
利益)和 *ʿurf*(习惯实践)。

像 pramāṇa 一样,象限旋转了 +π/4(§θ-fix-1, 2026-05-18),以清除垂直
轴上的 Stratum 标签 — 因此几何位置现在是 E/S/W/N 而非最初记录的
NE/SE/SW/NW。

## 范围

**何时使用此传统。** 当处理或可被分析为逊尼派伊斯兰法律学术推理的
内容时。对查看推导中证明类型的平衡很有用:你的论点是否严重以古兰经
为基础?它是否依赖共识?qiyās 是否承担了大部分工作?四个扩展芯片是
视觉提醒,经典 uṣūl 承认四个标题来源之外的更多内容。

**何时不使用此传统。** 对非伊斯兰内容,象限标签没有意义。框架也专门
是逊尼派 — 十二伊玛目什叶派 uṣūl 用 ʿaql(理性)替换 qiyās,并根据
宗教谱系规则(orientation v2.09)被有意排除。神秘主义、哲学和文学
内容不适合。

## 适用性

- 逊尼派 fiqh 推导、*uṣūl al-fiqh* 课程、教法判决分析。
- 法律学术写作中的跨来源平衡审计。
- 教授经典伊斯兰法学的证明类型结构。

## 谱系

经典逊尼派 uṣūl al-fiqh — 伊斯兰法律推理来源和方法的学科。四来源
正典在四个逊尼教派(哈乃斐、马立克、沙斐仪、罕百里)间是常规的,
每个来源如何被权衡有内部变化。Constellation 渲染遵循 al-Ghazālī
《*Mustaṣfā*》路线。

## 批评

将 ijmāʿ 放在 *ijtihādī*(推理派生)集群而非 *naṣṣ*(文本传递)集群
是有争议的,Ashʿarī/Māturīdī kalām 将 ijmāʿ 视为约束性传递。
Constellation 提供 Mustaṣfā 对齐的解读;替代 kalām 解读是 v4.1 打磨
目标。四来源正典也平坦化了四个教派之间的教义差异 — 哈乃斐特定或
马立克特定的变体寄存器可以稍后添加。

什叶派 uṣūl 的排除是产品设计选择(orientation v2.09 的宗教谱系规则),
而非学术判断。

## 引用

**一手资料。** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*,
ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

**现代。** Franz Rosenthal, *Knowledge Triumphant: The Concept of
Knowledge in Medieval Islam* (Leiden: Brill, 1970); Wael B. Hallaq,
*A History of Islamic Legal Theories* (Cambridge: Cambridge University
Press, 1997).

## 每条笔记的 frontmatter

`masadir_source: quran | sunnah | ijma | qiyas`。当 Rust 端的
`LayoutCacheRow` 扩展登陆时,此字段将覆盖默认放置(目前所有笔记 →
古兰经)。通过 `istihsan | istishab | maslaha | urf` 为扩展芯片来源的
每条笔记 opt-in 是后续工作。
