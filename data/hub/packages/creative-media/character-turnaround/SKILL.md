---
name: character-turnaround
description: Create prompt-only four-view character turnaround sheets from one or more reference images. Use when a user asks for front, 45-degree side, profile, and back views, character design sheets, identity locking, or reusable character prompts. The skill outputs copy-ready prompts and never generates images directly. Do not use for LoRA training, model fine-tuning, seed management, or external image-generation accounts.
---

# 角色四视图提示词

把角色参考图整理成一条可直接交给图像模型的四视图设定板提示词。固定视图从左至右为：正面、45°侧前、正侧面、背面；取景可选“带肩头像”“身体服饰”“全身像”。本技能只输出提示词，不调用图像生成、编辑或视觉验收工具。

## 工作原则

1. 先统一身份，再改变视角或场景。不要从单一人像参考直接跳到复杂环境和新动作。
2. 四个视图必须写在同一条提示词中，要求同一角色、同一套服装、同一画风和同一光线；不要拆成四条互相漂移的提示词。
3. 参考图中的身份、发饰、服装和风格分别提取：保留可辨识设计；背景、姿势和场景不继承，除非用户明确要求。
4. 参考图未展示的背面或遮挡细节只能依据已见材质、图案和结构合理推断，不声称复原未知细节。
5. 不承诺跨图绝对一致。身份锚点块必须在同一任务的后续提示词中逐字复用，只替换视角、动作或场景段。

## 1. 提取身份锚点

阅读所有参考图。若参考图是本地文件，先用图像查看工具检查其可见信息；这些图只承担身份与设计参考，不承担背景或构图参考。

静默整理以下不可漂移项，并将其写成固定的“身份锚点块”：

- 脸部：脸型、眼型、虹膜颜色、眉形、鼻口比例、年龄感与可辨识标记。
- 头发：主色、长度、分缝、刘海、束发结构、发饰位置，以及不应改变的发束逻辑。
- 体态：体型、身高感、二次元头身比例与默认站姿。
- 服装：轮廓、主辅色、层次顺序、关键图案、领口、腰封、袖口、衣摆、鞋履与饰物的归属位置。
- 风格：用户锁定的 Style lock；若未锁定，只描述参考图中可迁移的媒介、线条、渲染和光色。

用户要求查看提示词或身份锚点时，完整展示该块；否则可直接将它嵌入最终提示词。用户已给出 Style lock 时，必须原样放在 `Identity anchor` 之前，不加入与其冲突的媒介、写实度或光色词。

## 2. 选择取景档位

先询问用户取景；用户未指定时默认 `F3`，因为它最完整地呈现服装结构和四个转面。三档取景不可混用：

| 代码 | 取景 | 统一裁切范围 | 重点 |
| --- | --- | --- | --- |
| F1 | 带肩头像 | 头顶至肩下，四个视图的肩线都可见 | 脸部、发型、发饰、领口与肩部结构 |
| F2 | 身体服饰 | 头顶至膝上 | 躯干服装、领口、袖口、腰线、腰封与下摆结构 |
| F3 | 全身像 | 头顶至脚底 | 完整体态、衣摆、手部、腿部、鞋履与背面轮廓 |

可用以下简短选择语句，不把取景代码写入身份锚点块：

```text
请选择四视图取景：F1 带肩头像／F2 身体服饰／F3 全身像（默认 F3）。
四个固定视角为：正面、45°侧前、正侧面、背面。
```

## 3. 固定四视图顺序

无论选择哪一档取景，提示词必须明确要求同一画布、同一角色、从左至右的以下顺序：

1. **正面**：面向观者，双眼与肩线正对镜头；F3 时双臂自然下垂、双脚和鞋履完整可见。
2. **45°侧前**：角色向右转约 45°，鼻梁、双眼、前后肩关系和服装层次清楚可读；不是纯侧面。
3. **正侧面**：严格右侧 90°，只保留一个侧面轮廓；鼻梁、发型体积、胸背线、腰线和衣摆侧轮廓清晰。
4. **背面**：严格背对观者，头部不回望；展示发型后部、后领、背部结构、腰封、衣摆和鞋跟。F1/F2 仍保持对应裁切范围。

四视图使用一致的相机高度、人物比例、站姿逻辑、光向和背景层。禁止把 45°侧前误画成正侧面，禁止添加第五个视图、镜像重复或不同角色。

## 4. 选择设定集画面语言

需要页面组织时，先阅读 [presentation-catalog.md](references/presentation-catalog.md) 选择 R1-R4，再阅读 [frame-style-map.md](references/frame-style-map.md) 匹配边框语言。它们只是提示词组件，不是外部模板、SVG、PNG 或布局参考图。

若用户未指定页面组织：后续要复用角色时用 `R1`；要展示角色气质时用 `R2`；需要世界观归属时用 `R3` 或 `R4`。始终提醒取舍：`R1` 四视图等权、最利于身份锁定；`R2-R4` 主视图更大，但辅助视图空间更紧。若用户只要干净四视图，可省略装饰性设定集语言，保留低对比中性背景。

## 5. 输出一条完整提示词

最终只交付一条可复制的完整提示词；不要调用图像工具，不要直接出图。提示词采用以下结构，并将 `[F1/F2/F3]`、`[R1-R4]` 和方括号内容替换为用户选择与已提取信息：

```text
Use case: stylized-concept character turnaround prompt.
Input images: Image 1 is the identity and design reference only; do not copy its background or pose.
Framing: [F1 带肩头像 / F2 身体服饰 / F3 全身像]. Use the same framing crop in all four views.

[Style lock, if provided]
Identity anchor:
[固定身份锚点块，后续提示词逐字复用]

Character-sheet composition:
[R1-R4 页面组织片段，或“clean four-view sheet with equal-width panels”]

Frame language:
[与当前画风匹配的边框片段；若无需边框则写“minimal unobtrusive separators”]

Background and atmosphere:
[低对比背景与空气层；不得遮挡人物]

Create one finished four-view character turnaround sheet on a single canvas. From left to right, show exactly the same character and exactly the same outfit in this order: front view facing the viewer; right-front 45-degree three-quarter view, clearly not a profile; strict right-side 90-degree profile; strict back view facing away without looking back. Apply the selected [F1/F2/F3] framing consistently to every view. Keep identical face, hair architecture, hair ornaments, costume construction, palette, age, body proportion, camera height, standing-pose logic, and controlled light across all four views. Make every required feature readable within the selected crop. Do not add a fifth view, extra people, mirrored duplicates, alternate outfits, cropped framing beyond the selected crop, readable text, labels, numbers, logos, watermark, UI, weapons, or unrelated props. The background and frame are part of the illustration, remain low contrast, and never cover the face, shoulders, clothing structure, silhouette, hands, feet, footwear, hair ornaments, or costume edges.
```

若用户没有参考图，先明确这是原创角色设计，不是既有人物的身份锁定；仍按同一四视图和取景档位输出提示词。若用户要求场景图，使用已确认的四视图设定板作为唯一身份与服装参考，并另交付一条场景提示词：

```text
Input image role: the approved four-view turnaround sheet is the sole identity and costume reference.
Identity anchor:
[与设定板完全相同的固定身份锚点块]
Scene:
[环境、动作与叙事瞬间]
Composition:
[镜头距离、视角、景别、人物与环境的遮挡顺序]
Constraints:
Keep the same face, hair architecture, hair ornaments, costume silhouette, palette and body proportion. Do not redesign the character. No extra people, text, logo, or watermark.
```

## 6. 提示词交付前自检

不生成图片，因此只检查提示词是否完整：

- 是否明确写出 F1/F2/F3 取景及同一裁切范围。
- 是否按“正面、45°侧前、正侧面、背面”写出四个视图，且 45°侧前明确不是纯侧面。
- 是否要求同一身份、服装、比例、画风、光线和单画布，并禁止第五视图与重复人物。
- F1 是否强调肩线与领口，F2 是否覆盖腰线与服装主体，F3 是否要求头顶至脚底及鞋履。
- 是否保留足够的身份锚点，且没有把未知细节写成确定事实。
- 是否只输出提示词（必要时附 2-4 条短的可纠偏提示），没有声称已生成或验收图片。

## 输出规范

- 用户要求“提示词”或调用本技能时：只交付一条完整四视图提示词；可在其后附 2-4 条针对性的可纠偏短句。
- 用户未选择取景时：使用 F3，并在提示词前用一句话标明默认取景，允许用户改成 F1 或 F2 后重新输出。
- 用户要求“出图”时：说明本技能当前只提供提示词，不自动生成；随后仍交付可复制提示词。
- 用户要求 LoRA、训练、种子或第三方服务时：说明本技能不覆盖这些流程，继续提供内置图像模型可使用的四视图提示词。
