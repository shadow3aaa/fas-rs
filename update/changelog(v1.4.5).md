# fas-rs(v1.4.5)

[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志

- 修复1.4.0zygisk分析线程绑定的一处错误
- 优化安装引导
- 优化逻辑
- 更新依赖库到最新版本


## 运行要求

- soc平台无要求
- Android12以上
- zygisk开启并且版本v4以上(magisk v26.0以上并且开启zygisk / ksu + zygisk-next)

## 特殊说明

- 对开启fas的游戏使用shamiko会导致不生效
- 采用zygisk注入劫持libgui获取frametime，存在部分被检测风险
