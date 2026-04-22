# 精选发现（Findings）

本文从 `logs/raw-journal.md` 提炼高价值观察，统一用于结论归纳与后续复现实验对比。

## 记录规范

每条发现使用同一结构：

- `id`：`F-XXX`
- `tags`：问题标签（可多选）
- `severity`：1-5
- `model_behavior`：`proactive` / `prompted` / `missed`
- `rounds`：修正轮次
- `evidence`：代码位置、测试输出或日志片段
- `implication`：对工具或流程的可迁移启示

## 标签集

`#ownership` `#lifetime` `#union` `#fnptr` `#error` `#test-diff` `#idiom` `#float` `#recursion` `#prompt`

## 条目模板

### F-XXX 标题

- `tags`：
- `severity`：
- `model_behavior`：
- `rounds`：
- `evidence`：

**Observation**  
（客观现象，避免结论先行）

**Analysis**  
（失败原因或成功机制）

**Implication**  
（对后续翻译任务或工具设计的可执行建议）

## 纳入标准

进入本文件的观察至少满足一项：

1. 具有可复现性（不同任务可能重复出现）。
2. 与直觉显著不一致（反例价值高）。
3. 能直接转化为流程改进或工具需求。
