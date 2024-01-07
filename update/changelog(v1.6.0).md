# fas-rs(v1.6.0)

[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志

- 逻辑: fas延迟10s启动/恢复，以避免目标fps不准确时就执行
- 逻辑: 优化代码可读性，重新思考框架逻辑，增加稳定性
- 逻辑: 删除clap参数解析库改为手动解析，切换log输出库为flexi_logger以减小模块体积
- 编译: 切换优化等级为Os，降低模块大小
- 维护: 更新依赖包到最新版本

## 运行要求

- soc平台无要求
- Android12以上
- zygisk开启并且版本v2以上(magisk v24.0以上并且开启zygisk / ksu + zygisk-next)

## 特殊说明

- 对开启fas的游戏使用shamiko等隐藏可能会导致不生效, 是否生效以/sdcard/Android/fas-rs/fas_log.txt是否有对应游戏记录为准
- 采用zygisk注入劫持libgui获取frametime，存在部分被检测风险
