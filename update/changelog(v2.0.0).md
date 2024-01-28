# fas-rs(v2.0.0)

[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志

- [插件系统](https://github.com/shadow3aaa/fas-rs-extension-module-template)
- 更新依赖
- 修改编译参数，提高执行效率

## 运行要求

- soc平台无要求
- Android12以上
- zygisk开启并且版本v2以上(magisk v24.0以上并且开启zygisk / ksu + zygisk-next)

## 特殊说明

- fas boost模式是专门用于提升帧率的模式, 它不会尝试限制最大频率, 而是尝试在卡顿时提升最小频率以减少默认调速器的掉帧, 此模式不可通过帧率曲线来判断是否生效
- 对开启fas的游戏使用shamiko等隐藏可能会导致不生效, 是否生效以`/sdcard/Android/fas-rs/fas_log.txt`是否有对应游戏记录为准
- 采用zygisk注入劫持libgui获取frametime, 存在部分被检测风险

___

# fas-rs(v2.0.0)

[Homepage](https://github.com/shadow3aaa/fas-rs)

## Change log

- [Extension system](https://github.com/shadow3aaa/fas-rs-extension-module-template)
- Update dependencies
- Modify compilation parameters to improve execution efficiency

## Running requirements

- No requirements for soc platform
- Android12 or above
- zygisk is enabled and version v2 or above (magisk v24.0 or above and zygisk / ksu + zygisk-next is enabled)

## Special Instructions

- The fas boost mode is a mode specifically used to increase the frame rate. It does not try to limit the maximum frequency, but tries to increase the minimum frequency when stuck to reduce frame drops of the default governor. This mode cannot be judged by the frame rate curve. Is it effective?
- Using shamiko and other hiding methods on games with fas enabled may not take effect. Whether it takes effect depends on whether there is a corresponding game record in `/sdcard/Android/fas-rs/fas_log.txt`.
- Using zygisk injection to hook libgui to obtain frametime, there is some risk of being detected by game.
