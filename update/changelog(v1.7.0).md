# fas-rs(v1.7.0)

[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志

- 逻辑: 调整框架逻辑
- 逻辑: 调整计算平均偏差时机，给出更高的acc_dur上限以增加频率稳定性
- 逻辑: 调整topapp更新周期
- 逻辑: cpu控制器采用统一的频率变化量
- 构建: 自动构建添加debug版本
- 维护: 更新依赖包到最新版本

## 运行要求

- soc平台无要求
- Android12以上
- zygisk开启并且版本v2以上(magisk v24.0以上并且开启zygisk / ksu + zygisk-next)

## 特殊说明

- 对开启fas的游戏使用shamiko等隐藏可能会导致不生效, 是否生效以/sdcard/Android/fas-rs/fas_log.txt是否有对应游戏记录为准
- 采用zygisk注入劫持libgui获取frametime，存在部分被检测风险
