---
id: mohist-san-biao
name: Mohist sān biǎo
family: chinese-pragmatist
shape: horizontal-bands
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# 墨家 三表 (Mohist sān biǎo)

**家族**:中国实用主义 · **形状**:水平带(3 个区域)

## 核心隐喻

穹顶分为**从上到下堆叠的三个水平区域**,每个区域对应墨家评估学说的
一个标准:

- **本 běn** — 顶部。圣王的历史先例:学说在继承的传统中有保证吗?
- **原 yuán** — 中部。直接观察证据:普通人是否看到并听到它是这样的?
- **用 yòng** — 底部。实际社会利益:采用这种学说是否改善人民的生活?

只有通过所有三个测试,学说才值得持有 — 但 Sight 渲染让你看到分布在
三个区域的笔记,以感受 Universe 中哪种保证类型做了最多工作。

水平轴不携带特定编码 — 墨家的三个标准是*范畴的*,而非序数的,因此带
内的定位通过确定性的每条笔记抖动。

## 范围

**何时使用此传统。** 当处理测试是*学说是否值得持有*的内容时,而非
什么样的保证支撑它。对政策、伦理、应用经验和实际决策内容很有用,
其中历史先例 / 观察 / 利益是辩护的三个轴。

**何时不使用此传统。** 当内容没有学说或评估维度时。纯粹描述性内容、
创造性工作和关于主观经验的笔记不适合。

## 适用性

- 政策提案及其辩护。
- 比较伦理分析(此规则是否通过三个测试?)。
- 利益人民明确的工程和应用科学。

## 谱系

经典中国实用主义认识论。墨子(Mòzǐ,约公元前 5 世纪)创立了墨家学派,
将自己呈现为对儒家的批判性替代。三表出现在"非命"章节中,作为墨家
对继承的宿命论学说应用的测试 — 并得出结论它失败了所有三个测试。
学派短暂繁荣后被儒家和法家的兴起所掩盖;它作为可恢复的正典文本生存
下来,今天通过《*墨子閒詁*》等版本进行研究。

## 批评

三表有时被批评为混淆了证据保证与效用的早期实用主义形式 — 特别是
"利益人民"标准很难形式化。现代学者也争论三表是一个完全发展的认识
论理论,还是在特定反宿命论论证中部署的论战修辞工具。尽管有天神学
背景,但因方法论核心是世俗的,根据宗教谱系规则被祖父条款纳入策划
基线。

## 引用

**一手资料。** *墨子* 第 IX 卷,"非命上"。批判版:孙诒让 编,
*墨子閒詁*,2 vols.(北京:中华书局,1986)。英译:Ian Johnston,
trans., *The Mozi: A Complete Translation* (New York: Columbia University
Press, 2010).

**现代。** A. C. Graham, *Disputers of the Tao: Philosophical
Argument in Ancient China* (La Salle, IL: Open Court, 1989), ch. 1;
Chris Fraser, "Mohism," *Stanford Encyclopedia of Philosophy* (2020).

## 每条笔记的 frontmatter

`mohist_zone: ben | yuan | yong`。当前缺失 — 笔记按 notePath 确定性
地哈希分桶到三个区域中,因此视觉结构得以填充。当 Rust 端的
`LayoutCacheRow` 扩展登陆时,此字段将覆盖哈希分桶分配。
