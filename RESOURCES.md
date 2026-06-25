# Shader 学习资源

## Knowledge

- [W3C: WebGPU Shading Language specification](https://www.w3.org/TR/WGSL/)
  WGSL 的权威语言规范。用于查询 shader stage、入口函数、类型、地址空间、binding 和内置函数。
- [Bevy: Shader examples](https://bevy.org/examples/shaders/)
  官方可运行示例，展示 Bevy 如何把 Rust 侧材质及渲染管线连接到 shader。用于查找项目集成模式。
- [Bevy 0.19: Material using WESL](https://github.com/bevyengine/bevy/blob/v0.19.0/examples/shader/shader_material_wesl.rs)
  与项目版本匹配的第一方示例，用于 WESL shader 加载和被导入模块的生命周期管理。
- [Bevy 0.19: Fullscreen vertex shader](https://github.com/bevyengine/bevy/blob/v0.19.0/crates/bevy_core_pipeline/src/fullscreen_vertex_shader/fullscreen.wgsl)
  当前全屏材质所用顶点入口的第一方源码。用于确认可见视口中的 UV 方向、范围和插值来源。
- [WESL compiler documentation](https://docs.rs/wesl/0.3.2/wesl/)
  Bevy 0.19 所用 WESL 版本的文档。用于查询 import、模块、条件翻译和编译器行为。
- [Encase 0.12: `ShaderType`](https://docs.rs/encase/0.12.0/encase/trait.ShaderType.html)
  Bevy 0.19 用来把 Rust 数据编码为 shader buffer layout 的底层库文档。用于核对 uniform 结构、对齐和编码约束。
- [Bevy ECS 0.19: `Component`](https://docs.rs/bevy_ecs/0.19.0/bevy_ecs/component/trait.Component.html)
  与项目版本匹配的组件权威 API 文档。用于区分游戏侧效果数据与 GPU material 传输结构。
- [Bevy Time 0.19: `Time`](https://docs.rs/bevy_time/0.19.0/bevy_time/struct.Time.html)
  与项目版本匹配的时间 API 文档。用于区分基于 elapsed time 的直接求值动画与基于 delta time 的积分更新。
- [`region_color_screen_material.wesl`](assets/shaders/region_color_screen_material.wesl)
  项目中正在使用的全屏效果，作为阅读、修改和实验的主要样本。

## Wisdom (Communities)

- [Bevy Discord](https://discord.gg/bevy)
  官方实时社区。用于验证 Bevy 特有的 shader 集成假设和平台行为。
- [Bevy GitHub Discussions](https://github.com/bevyengine/bevy/discussions)
  可搜索的引擎贡献者讨论。用于设计权衡和渲染架构问题。
- [GPUWeb GitHub Discussions](https://github.com/gpuweb/gpuweb/discussions)
  关注标准的社区。用于不局限于 Bevy 的 WGSL 和 WebGPU 语义问题。

## Gaps

- 初次设置时在线检索返回 HTTP 403。以上链接均为权威来源；课程中的陈述已与本机安装的 Bevy 0.19 和 WESL 0.3.2 源码交叉核对。
