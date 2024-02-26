# fas-rs(v2.2.0)

[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志

- 频率调整步进自动改变
- 默认配置不开启fas_boost
- 加回fas自动暂停分析
- 优化频率稳定性
- 优化二进制大小(编译标准库)

## 运行要求

- soc平台无要求
- Android12以上
- zygisk开启并且版本v2以上(magisk v24.0以上并且开启zygisk / 其它su + zygisk-next)

## 特殊说明

- fas boost模式是专门用于提升帧率的模式, 它不会尝试限制最大频率, 而是尝试在卡顿时提升最小频率以减少默认调速器的掉帧, 此模式不可通过帧率曲线来判断是否生效
- 对开启fas的游戏使用shamiko等隐藏可能会导致不生效, 是否生效以`/sdcard/Android/fas-rs/fas_log.txt`是否有对应游戏记录为准
- 采用zygisk注入劫持libgui获取frametime, 存在部分被检测风险
