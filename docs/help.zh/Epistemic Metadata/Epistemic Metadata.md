# Epistemic Metadata

> **翻译说明:** 本帮助主题为 AI 生成的翻译,源自 `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md` 中的英文规范版本。母语者审阅尚未完成。请通过项目仓库提交修正。

*(MIG-022 §A — 差距分析 §6.1 模式扩展)*

本主题描述 Constellation 现在可识别的一小组 **可选前置元数据字段**,用于对笔记进行更丰富的认知分类。这些字段是为响应差距分析(`docs/epistemic-content-gap-analysis.md`)而添加的 — 即认识到 Constellation 认知内容引擎(CECE)用于分类的"来源 × 内容类型"双轴模型并不能涵盖你想要记录的关于"如何获知"的所有内容。

这些字段 **全部可选**。没有它们的现有笔记照常工作。当一篇笔记是从额外信号中受益的那种知识时,你可以手动(或将来通过结构化编辑器)添加它们。

---

## 字段列表

### `held_by` — *这是谁的立场?*

一个简短字符串,指明持有该笔记所述立场的人。默认为 `user`(你自己的立场)。可以使用的其他值:
- 学者姓名: `held_by: "al-Shāfiʿī"`
- 学派 (madhhab): `held_by: "Ḥanafī"`
- 历史人物: `held_by: "Aristotle"`

当你写一篇记录 *他人* 立场而非你自己立场的笔记时,`held_by` 是表明这一点的字段。没有它,Constellation 默认假定笔记的认知状态是你自己的 — 对于严肃的学术工作来说,这往往是错误的。

### `domain` — *这是关于什么主题的?*

学科标签列表。与自由形式的 `tags` 字段(大众分类法 / 心情 / 项目)不同,`domain` 是用于检索和过滤的结构化学科/主题字段。例:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

被分类为 `content_type: "proposition"` 和 `source: "inference"` 的笔记可能是逻辑定理(domain: `[logic, mathematics]`),也可能是法律意见(domain: `[fiqh, ʿibādāt]`)— 相同的认知形态,却是非常不同的检索语境。`domain` 让你可以指明是哪一种。

### `function` — *这篇笔记是干什么的?*

标识笔记预期用途的单一字符串。可识别的值:

- `reference` — 需要时阅读(定义、引用、以后查阅的事实)
- `seed` — 孵化(仍在发展中的早期阶段想法)
- `actionable` — 用它做点什么(任务、跟进、要做的决定)
- `shipped` — 完成品(已发布的文章、已交付的分析、闭合的循环)

不同于 CECE 的内容类型轴(说明它是哪种知识)— `function` 说明你 *将* 用这篇笔记做什么。

### `provenance_civilization` — *哪个传统的词汇在起作用?*

一个可选字符串,标识笔记词汇的文明足迹。对于针对特定传统语料库的检索很有用。例:

- `provenance_civilization: "sunni-usuli"` — 逊尼派 *uṣūl al-fiqh* 传统(al-Bukhārī、al-Ghazālī、al-Āmidī)
- `provenance_civilization: "analytic-western"` — 弗雷格之后的分析哲学
- `provenance_civilization: "nyaya"` — pramāṇa 认识论的印度 Nyāya 学派
- `provenance_civilization: "buddhist-pramana"` — 佛教认识论传统(Dignāga、Dharmakīrti)

大多数笔记不需要这个。当你有一篇同时借鉴了逊尼派 *uṣūl* 和分析英美认识论的笔记时,记录主要足迹有助于未来的你检索到正确的可比材料。

### `updated_at` — *你的立场上一次改变是什么时候?*

笔记认知内容最近一次有意修订的 ISO 日期。不同于文件系统的 `modified` 时间戳(每次保存都会捕获,即使是修复错别字);`updated_at` 是当你实际重新思考立场时 *你* 设定的时间戳。

```yaml
updated_at: 2026-05-09
```

当 §6.3 时间轴的其余部分(笔记状态历史)完成时很有用 — 在那之前,这是一个单快照字段,记录"我最后一次修订观点的时间"。

### `ikhtilāf` — *结构化的学术分歧*

新字段中最复杂的一个。将 *ikhtilāf* — 学者或学派对某个问题的结构化分歧 — 记录为 `{school, position}` 对的列表。Constellation 提供一个用于编辑此项的自定义 Properties 面板小部件;你也可以直接编辑 YAML。

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

带有 `ikhtilāf` 的笔记不处于任何单一的认知状态 — 它记录了多个行为者之间的 *结构化分歧*。没有此字段,Constellation 会将这种笔记视为它自身持有这些立场之一,这是错误的。

Properties 面板将每行渲染为带有两个输入(school + position)和一个删除按钮的编辑器卡片,底部带有"添加学派"按钮。

### `warrant` 和 `warrant_notes` — *已解析但 (暂时) 不活跃*

两个字段被解析并存储到磁盘,但 **尚未在任何 UI 中显示**:

- `warrant: "mutawātir"` — 笔记主张依据(warrant)的等级标签。逊尼派 *uṣūl* 等级使用 *mutawātir / mashhūr / āḥād*,在 hadith 内部专门使用 *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*。其他传统有自己的等级词汇。
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — 支持 warrant 等级的自由文本。

当 **Constellation Warrant Research 工作流** 发布其分类器时(数月研究项目;参见差距分析 §6.2),这些字段就准备好使用了。在那之前你可以手动填写,数据会持久保存;没有任何东西显示它。未来的 warrant 感知查询和徽章会直接读取这些值。

---

## 这些字段出现的位置

当你在笔记的前置元数据中填入任何新字段时,它们会以与所有其他 YAML 字段相同的方式出现在 **Properties 面板**(右侧栏)中 — 每个键一行,配以适合类型的编辑器:

- `held_by`、`function`、`provenance_civilization`、`warrant`、`warrant_notes` → 文本输入
- `domain` → 标签列表(输入 + Enter 添加,各标签上的 × 删除)
- `updated_at` → 日期选择器
- `ikhtilāf` → 带有 `school` / `position` 行 + 添加/删除按钮的自定义小部件

---

## `supersedes` 怎么处理?

从技术上讲,`supersedes` 是 *笔记之间的关系*,而不是单个笔记的属性。Constellation 将其作为 **类型化链接** 处理,而不是 YAML 标量:

```markdown
This note replaces my earlier analysis: [[old-note-id|supersedes]]
```

wikilink 上的 `|supersedes` 后缀告诉 Constellation 这是 `supersedes` 类型的类型化链接 — 它有独特的胶囊颜色(板岩蓝灰),与其他类型化链接一起出现在 Backlinks + Outgoing Links 面板中,并参与 Living Link Architecture(权重、生命周期、遍历计数)。

这将笔记到笔记的关系保持在一个地方 — 类型化链接系统 — 而不是将它们分散在类型化链接和前置元数据标量之间。`contradicts:` 也适用同样原则(在 MIG-022 之前的词汇中已经是类型化链接)。

---

## 这 *不是* 什么

这些字段当前 **不被** CECE 分类所消费。CECE 仅按"来源 × 内容类型"分类;新元数据字段是为人工驱动的检索、未来的 warrant 感知分类器以及时间轴(当它发布时)而记录的。

特别是:
- `function: "actionable"` *不会* 在 Tasks 面板中自动创建任务
- `held_by: "al-Shāfiʿī"` *不会* 改变 CECE 对笔记的分类方式
- `domain: [fiqh]` *不会* 过滤你的搜索结果,除非你将其包含在搜索查询中

这些字段是 **模式** — 你可以添加的可识别词汇。未来的 MIG 将发布消费它们的功能(warrant 分类器、时间查询、领域感知过滤等)。

---

## 工作示例

记录逊尼派学派关于"破晓断食义务对当日有效性是否重要"立场的笔记:

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

七个新字段中填了六个;`warrant_notes` 省略(暂无传承细节可记);`supersedes` 和 `derives-from` 作为正文中的类型化链接,而非 YAML 标量。

---

*MIG-022 §A — 此模式扩展随本 Constellation 构建发布。Warrant Research 工作流(独立 Concept Paper,数月)将发布消费 `warrant` 字段的 warrant 分类器。时间轴(MIG-023,独立 Architect 周期)将消费 `updated_at` 加上更广泛的笔记状态历史。*
