# 使命：阅读并编写游戏效果 Shader

## Why
把 shader 作为在 `dungeonslime_clone` 中实现视觉效果的实用工具，而不是将其视为不透明的渲染基础设施。

## Success looks like
- 阅读项目中的 shader，并解释其输入、变换和输出。
- 在运行前预测一处小型 WESL/WGSL 修改会产生什么画面结果。
- 在 Bevy 中独立实现并调试简单的 2D 材质和全屏效果。

## Constraints
- 课程基于本项目的 Bevy 0.19 与 WESL shader 代码。
- 从用户已有的函数式编程心智模型出发。
- 优先采用短课，并提供可立即检验的结果。

## Out of scope
- 在建立基础 shader 熟练度之前，暂不展开高级 GPU 性能调优和自定义 Render Graph 设计。
