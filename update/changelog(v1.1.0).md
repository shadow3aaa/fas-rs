# fas-rs(v1.1.0)
[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志
- 在release max的时候恢复调速器，以减少无法避免
的卡顿的影响
- 调整参数 & 更新依赖
- 优化策略
- 分离时间累积器
- 优化构建系统，修复在非termux环境实际不能构建的问题

## 运行要求
- soc平台无要求
- Android12以上
- zygisk开启并且版本v4以上(magisk v26.0以上并且开启zygisk / ksu + zygisk-next)

## 特殊说明
- 对开启fas的游戏使用shamiko会导致不生效
- 采用zygisk注入劫持libgui获取frametime，存在部分被检测风险
