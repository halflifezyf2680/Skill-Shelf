# 完整闭环流程

`writer-studio` 有两条闭环：

- 项目初始化闭环
- 章节写作闭环

不要把两者混成一条。

## 一、项目初始化闭环

适用于项目还没有正式 truth surface，或者只有散乱文档的时候。

### 目标

- 建立统一的数据结构维护协议
- 建立作者维护面
- 建立数据库查询面
- 建立脚本层
- 让后续写作闭环可运行

### 顺序

1. 运行 `writer-status`  
   先确认当前阶段和缺口

2. 明确当前项目的 truth surface 形态  
   默认是：结构化 YAML / 结构化文档块

3. 定义候选到 canon 的准入机制  
   先把 brainstorm、candidate、canon 分开

4. 运行 `writer-init`，定义结构化数据维护协议  
   明确字段、主表 / 子表语义、稳定 id、查询能力

5. 生成项目脚本计划  
   至少包括扫描、校验、落库、查询、视图构建

6. 运行 `writer-canonize`，建立第一版 truth surface  
   开始把正式 canon 以协议规定的格式写入 YAML / 结构化文档

7. 编译出第一版数据库查询面  
   数据库不是作者面，而是编译产物

8. 生成基础派生视图  
   至少应能生成树状视图、关系查询结果、章节上下文输入

### 完成标准

- 项目已经有正式 truth surface
- YAML 与数据库字段同构
- 脚本已能扫描、校验、落库、查询
- 候选方案不会再直接污染 canon

## 二、章节写作闭环

适用于项目已经有正式 truth surface，可以开始稳定写章。

### 目标

- 用 packet 驱动正文
- 用 audit 发现漂移
- 用 sync 维护 canon

### 顺序

1. 运行 `writer-status`
2. 运行 `writer-build-packet`
3. 按 packet 起草 draft
4. 运行 `writer-audit`
5. 运行 `writer-sync`
6. 先更新 YAML / 结构化文档
7. 再通过脚本编译到数据库查询面
8. 重新生成需要的派生视图

### 完成标准

- 章节已写完
- 审校已通过
- canon delta 已处理
- truth surface 与查询面重新一致

## 三、统一原则

- 文档和 YAML 是作者维护面
- 数据库是查询与分析面
- 脚本负责二者之间的编译与校验
- 不允许文档一套规范、数据库另一套规范
- 任何新增结构都必须先体现在协议中，再进入长期维护

## 四、跨轮次经验传递（lessons-learned）

项目级 `lessons-learned.md` 是跨章节、跨轮次的经验积累文件。它不是可选的，是两条闭环的强制组件。

### 什么时候写

每次以下事件发生时，必须提取教训：

- `writer-audit` 返回 `fail` 或 `concerns`，且命中了有教学意义的问题
- 手动修正（fix-protocol）暴露了反复出现的模式
- coordinator 发现 subagent 犯了同类错误超过 1 次

### 写什么格式

```markdown
# Lessons Learned

## 致命项相关

### LL-001: 元叙述越界——解释性独白
- 首次发现：Chapter 3 audit
- 触发条件：角色面临陌生设定时，叙述倾向于"停下来解释"
- 防御措施：packet 的 forbidden_moves 中预填"禁止角色自言自语解释世界观"
- 审查检查点：F1（元叙述越界）

### LL-002: （后续补充）

## 文风相关

### LL-003: 情感空洞——动作场景缺失生理反应
- 首次发现：Chapter 5 audit
- 触发条件：高强度动作段落中，叙述变成纯动作描写流水账
- 防御措施：style_constraints 中提高情感密度下限，要求"动作 + 生理反应"交替
- 审查检查点：N5（情感空洞）

## 结构相关

### LL-004: （后续补充）
```

### 怎么用

以下环节必须读取 `lessons-learned.md` 作为额外约束：

1. **`writer-build-packet`** 构建章节执行包时
   - 读取已有教训，检查 forbidden_moves 和 style_constraints 是否需要补充防御措施
   - 如果项目已有教训文件，packet 构建报告中必须声明"已对照 lessons-learned"

2. **`writer-audit`** 审校正文时
   - 读取已有教训，检查本次 audit 是否暴露了已知问题
   - 如果已知问题再次出现，在 audit 报告中标注"重复命中 LL-XXX"

3. **coordinator spawn subagent** 调度子代理时
   - 将 `lessons-learned.md` 作为上下文附带给 subagent
   - 特别是 novelist subagent，必须携带教训清单

### 维护规则

- 每条教训必须有唯一编号（LL-NNN），方便跨文件引用
- 教训条目一旦写入，不得删除（历史可追溯）
- 如果某条教训不再适用（如项目风格发生了有意为之的转变），标记为 `[已覆盖]` 而非删除
- 建议每 5 章回顾一次教训文件，合并重复条目
