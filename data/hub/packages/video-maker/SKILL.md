---
name: video-maker
description: AI短视频制作全流程：分镜脚本生成、专业级AI生图提示词、素材管理、视频合成。基于MoneyPrinterTurbo后端。适用于科普、AI开发、技术教程类抖音/B站视频。触发词："做个视频"、"生成视频"、"视频脚本"、"生图提示词"、"分镜脚本"。
---

# Video Maker Skill

## 概述

基于 MoneyPrinterTurbo 的 AI 短视频全自动化制作流程。

核心理念：**素材质量决定视频质量**。文案+配音由AI生成，画面素材由AI生图提供，合成由工具完成。

## MoneyPrinterTurbo 服务

- 位置: `G:\Bot_WorkSpace\apps\MoneyPrinterTurbo`
- 启动: `cd G:\Bot_WorkSpace\apps\MoneyPrinterTurbo && .\.venv\Scripts\python main.py`
- API文档: http://127.0.0.1:8080/docs
- 配置: `G:\Bot_WorkSpace\apps\MoneyPrinterTurbo\config.toml`

## 完整流程

### 步骤1: 生成分镜脚本

根据主题生成结构化分镜脚本，每个场景包含：
- 旁白文案（每段15-25字，约3-5秒）
- 画面中文描述（给创作者审阅）
- 专业级英文生图提示词（直接给AI生图工具用）

### 步骤2: 生图提示词工程

提示词必须按专业摄影框架编写，包含以下层级：

#### 主体描述层
- 主要拍摄对象的详细描述（物品、场景、概念可视化）
- 具体属性、质感、材质
- 和环境的关系、空间位置

#### 环境与场景层
- 场景类型：影棚、户外、城市、自然、室内、抽象空间
- 环境细节：具体元素、纹理、天气、时间
- 背景处理：清晰、虚化、渐变、极简
- 大气条件：雾、尘、通透

#### 光线设定层
- 光源：自然光（黄金时段、阴天）或人工光（柔光箱、轮廓光、霓虹灯）
- 光线方向：正面、侧面、逆光、顶光、伦勃朗光
- 光质：硬光/柔光、漫射、体积光、戏剧性
- 色温：暖调、冷调、中性、混合光源

#### 技术摄影层
- 机位：平视、仰拍、俯拍、鸟瞰
- 焦距效果：广角畸变、长焦压缩、标准视角
- 景深：浅景深、大景深、选择性对焦
- 曝光风格：高调、低调、均衡、HDR、剪影

#### 风格与美学层
- 摄影类型：编辑、商业、艺术、信息图
- 后期处理：胶片模拟、调色、对比度、颗粒感
- 色彩基调：统一贯穿全视频

### 步骤3: AI生图/素材准备

将提示词发送给AI生图工具（nano banana等），生成对应画面素材。

素材存放: `G:\Bot_WorkSpace\apps\MoneyPrinterTurbo\storage\local_materials\{task_name}\`
命名: `01.jpg`, `02.jpg`, `03.jpg`（按分镜顺序）

### 步骤4: 合成视频

```powershell
$body = @{
    video_subject = "主题"
    video_script = "完整旁白文案"
    video_language = "zh"
    video_aspect = "16:9"
    voice_name = "zh-CN-YunxiNeural"
    video_source = "local"
    video_materials = @("G:\path\01.jpg","G:\path\02.jpg")
    subtitle_enabled = $true
    bgm_type = "random"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/videos" -Method POST -ContentType "application/json" -Body ([System.Text.Encoding]::UTF8.GetBytes($body))
```

## 分镜脚本 Prompt 模板

```
# Role: 视频分镜脚本生成器 + 图像提示词工程师

## 目标
根据主题生成完整分镜脚本。每个场景包含旁白文案和专业级AI生图提示词。

## 画面风格
科普/AI开发类视频，统一视觉风格：
- 现代简洁插画风格，科技感
- 软渐变背景，信息图式视觉
- 无人物特写（AI生图人脸不稳定）
- 无文字元素（字幕会叠加）
- 每张图画面独立成景，不依赖前后文

## 提示词规范
每条提示词必须包含以下层级，用英文编写：
1. 主体描述：具体、可视化，概念转为视觉元素
2. 环境/场景：背景类型和细节
3. 光线设定：光源类型、方向、光质、色温
4. 技术参数：机位、焦距、景深、构图
5. 风格修饰：后期风格、色彩基调、参考
6. 画幅参数：末尾加 --ar 16:9 或 --ar 9:16
7. 质量词：8k resolution, editorial quality, professional

## 约束
1. 输出为JSON数组格式
2. 每个场景旁白15-25字（约3-5秒）
3. 总场景数4-6个，总时长30-60秒
4. image_prompt_cn 是中文画面描述，供审阅
5. image_prompt_en 是英文专业提示词，给AI生图工具直接使用
6. 所有场景的视觉风格必须统一一致
7. 提示词中的光线方向、阴影方向必须物理上自洽

## 输出格式
严格输出JSON，不要任何其他内容：
[
  {
    "index": 1,
    "narration": "旁白文字",
    "image_prompt_cn": "中文画面描述",
    "image_prompt_en": "English prompt with full photography specs, --ar 16:9"
  }
]

## 主题: {subject}
## 画面比例: {aspect}
## 语言: 中文
```

## 配音音色

| 音色ID | 描述 |
|--------|------|
| zh-CN-YunxiNeural | 男声，年轻，适合科普 |
| zh-CN-YunjianNeural | 男声，成熟 |
| zh-CN-XiaoyiNeural | 女声，温柔 |
| zh-CN-XiaohanNeural | 女声，知性 |

## 视频比例

| 平台 | 比例 | 参数 |
|------|------|------|
| B站/YouTube | 16:9 | `video_aspect: "16:9"` |
| 抖音/快手 | 9:16 | `video_aspect: "9:16"` |

## 查询与下载

```powershell
# 查询任务
Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/tasks/{task_id}"

# 下载视频
Invoke-WebRequest -Uri "http://127.0.0.1:8080/api/v1/download/{file_path}" -OutFile "output.mp4"
```

## 注意事项

- 确保 MoneyPrinterTurbo 服务已启动（端口 8080）
- 本地素材模式: `video_source: "local"`
- 生图提示词避免文字、避免人脸特写
- 所有场景视觉风格要统一（色调、光影风格一致）
- 素材路径用绝对路径
