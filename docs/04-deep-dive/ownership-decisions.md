# 深度文章：所有权决策（Ownership Decisions）

本章以 ADR 方式记录所有权相关的高影响决策。

## 1. 决策目标

将 C 中隐式内存约定转化为 Rust 中可验证的类型约束，同时控制实现复杂度与性能成本。

## 2. 重点议题

- 表达式树的所有权模型（`Box`/`Rc`/Arena）
- 变量绑定的生命周期策略
- `unsafe` 的必要性与边界
- 错误传播与资源释放耦合关系

## 3. ADR 模板

### ADR-XXX 标题

- `status`：
- `date`：
- `context`：
- `options`：
- `decision`：
- `rationale`：
- `consequences`：

## 4. 评估原则

- 默认优先安全与可维护性，性能优化须以测量为依据。
- 每个决策必须配套回归测试或行为对照证据。
- 若引入 `unsafe`，需提供最小作用域和审计注释。
