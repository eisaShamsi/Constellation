---
aliases:
  - Constellation 心智
  - Constellation Mind
  - Mind
  - 本地 LLM
  - 本地大语言模型
  - Fanar
  - AI 聊天
  - 个人 AI
description: Constellation Mind 是 Constellation 的本地大语言模型(LLM)层——一个可以与之聊您自己笔记的 AI,完全运行在您的设备上。阶段 0b 已于 2026-05-24 发布,搭载从设置 → Mind 可安装的阿拉伯语优先模型 Fanar-1-9B。聊天界面将在阶段 1 落地。
---

# Constellation Mind (عقل Constellation)

## 这是什么?

Constellation Mind 是 Constellation 的本地大语言模型(LLM)层——一个了解您的宇宙并可以与您讨论您笔记的 AI 助手,**而不会将任何笔记发送到云端**。

三件事使它有别于其他每一个"笔记 AI"工具:

1. **本地优先。** 模型在您的设备上运行。您的笔记永远不会离开。没有云端往返——聊天是本地的并支持离线运行。
2. **阿拉伯语优先。** 捆绑的默认模型是 **Fanar-1-9B**,卡塔尔计算研究所(QCRI)以阿拉伯语为中心、具有逊尼派意识的模型。在 MSA 和海湾方言中具有母语水平能力;英语是第二语言,而非唯一语言。
3. **引用绑定。** AI 对您笔记所做的每一个事实性主张都必须引用源笔记。幻觉性引用会被生成后的验证器捕获(阶段 1)。

## 今天发布的内容(阶段 0b — 2026-05-24)

- **设置 → Mind 面板** — 列出可安装的模型(目前仅 Fanar 1.9B Q4_K_M,约 5 GiB),带有一个安装按钮可下载并验证模型。
- **模型安装** — 从 GitHub Release 分块下载(无第三方云),按块和组装后的整体进行 SHA-256 验证。
- **真实推理运行时** — `llama-cpp-2`(v1 中仅 CPU)加载 Q4_K_M GGUF 并流式传输令牌。
- **暂无聊天界面** — 那是阶段 1(下一个里程碑)。今天您可以安装并验证模型;对话 UI 将在 MIG-048 中落地。

## 如何安装 Fanar

1. 打开**设置 → Mind**。
2. 在目录中找到 **Fanar 1.9B (Q4_K_M)**。卡片显示大小(5.01 GiB)、许可证(带防御性 Gemma 通知的 Apache-2.0),以及"设为活跃"或"安装"按钮。
3. 点击**安装**。进度条分三个阶段显示下载 + SHA 验证 + 组装。
4. 当徽章切换到**已安装** + **活跃**时,模型已就绪。Fanar 位于 `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf`,并由 mmap 支持(不复制到 RAM)。

就是这样。在阶段 1 发布聊天界面之前,已安装的模型处于待机状态。

## 阶段 1 即将到来的内容(下一个里程碑)

- **聊天界面** — 一个 Constellation 面板,您可以用阿拉伯语或英语与 Fanar 谈论您的宇宙(每条消息支持 RTL)。
- **读取工具** — Mind 可以调用 `search_notes`、`read_note`、`find_similar`、`list_recent` 来将其回答根植于您的真实笔记。
- **引用验证器** — 每个主张都引用一个真实的笔记;编造的 `note:UUID` 引用在到达您之前就会被拒绝。
- **应用启动时预热** — Mind 在后台加载,这样您的第一次聊天就不需要支付 10 秒的冷加载。
- **对话历史** — 按宇宙保存;可提升为笔记。

完整架构请参见 `docs/Constellation-Mind-Concept-Paper-v1.1.md`,逐阶段路线图请参见 `docs/Constellation-Mind-Implementation-Plan-v1.0.md`。

## 稍后到来的内容

- **阶段 2 — 写入工具**(Mind 在您的明确批准下提议编辑 / 新笔记 / 链接)。
- **阶段 2.5 — RoutedProvider + Jais**(第二个模型,来自 G42/MBZUAI 的 Jais-2-8B,作为共同默认与 Fanar 一起加入;Mind 根据请求在它们之间进行路由)。
- **阶段 3 — 自动分类 + 智能链接**(Mind 在笔记保存时提议方面和链接)。
- **阶段 4 — 能力工具**(语音 → 笔记,OCR → 笔记,翻译)。
- **阶段 5 — 云端选用**(您自己的 Anthropic / OpenAI 密钥,带有每宇宙成本上限和每轮出口日志)。

## 隐私与数据流

- **仅在安装模型时出站 HTTP** — Constellation 从此仓库的 [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases) 下载模型文件。无遥测。无云端推理(尚无——那是阶段 5,且仅在您明确选用时)。
- **磁盘上:** 模型的 GGUF + 一个 `installed_models.json` 注册表,跟踪您拥有哪些模型以及哪个是活跃的。
- **运行时:** 加载的模型文件是内存映射的;您的提示和响应仅在 RAM 中存在。

## 许可证

每个模型在 GitHub Release 中都携带自己的 LICENSE.txt。对于 Fanar:

- **Apache License 2.0**(QCRI 在 Fanar-1-9B-Instruct 仓库上声明的许可证)。
- **Gemma 使用条款** — Fanar 是 `google/gemma-2-9b` 的持续预训练;即使 QCRI 仅将结果重新标记为 Apache-2.0,Constellation 也防御性地发布 Gemma 通知。
- **Fanar 引用**(Fanar Team 2025,arXiv:2501.13944)。
- **Constellation 再分发通知** — Constellation 的 GitHub Release 上的 GGUF 是 QCRI 上游 safetensors 的量化,由 `.github/workflows/model-pipeline.yml` 生成,并随原始 LICENSE 在 Apache-2.0 下分发。

完整的 LICENSE.txt 与每个模型一起存在于其发布中:<https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>。

## 故障排除

**"尚未就绪"徽章而不是安装按钮。** 捆绑的目录对该模型有一个占位符 SHA-256。在正常的 Constellation 安装上不应发生这种情况;如果您看到它,说明该模型版本的目录尚未更新。请提交一个 issue。

**安装在"下载部分 X/Y"处挂起。** 网络问题。从设置 → Mind 取消,重新触发安装——部分块会自动清理。

**安装成功,但文件 SHA-256 不匹配。** 下载时的位翻转。重新安装将获取新的副本。

**缺少聊天界面。** 阶段 1(MIG-048)尚未发布。今天可以安装并验证模型;对话 UI 将在下一个版本中落地。

---

*随着阶段 1 的发布,子主题将加入此文件夹:聊天 UI 演练、引用芯片点击行为、多模型选择器、第二屏幕上长聊天的渲染。*
