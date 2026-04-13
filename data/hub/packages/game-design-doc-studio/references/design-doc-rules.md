# 设计文档硬标准

- 每份系统设计文档必须至少覆盖：
  - Overview
  - Player Fantasy
  - Detailed Design
  - Formulas
  - Edge Cases
  - Dependencies
  - Tuning Knobs
  - Acceptance Criteria
- Formula 必须包含变量定义、范围、来源或含义
- Edge Cases 必须明确写出发生条件与结果，不能只写“妥善处理”
- Dependencies 必须具体到接口或依赖关系，不能只写“与 X 系统联动”
- Tuning Knobs 必须说明哪些值可调、调大会怎样、调小会怎样
- Acceptance Criteria 必须可验证，不能写抽象口号
- 不允许 hand-waving
- 平衡值必须能追溯到公式、机制逻辑或明确理由
- 长文档应先建骨架，再逐段补全

