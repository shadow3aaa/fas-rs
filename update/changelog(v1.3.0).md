# fas-rs(v1.3.0)

[项目主页](https://github.com/shadow3aaa/fas-rs)

## 更新日志

- scheduler逻辑大重构，以增加不同场景的通用性
- 多模式设置不同水位线
- 优化参数
- 优化构建框架

## 运行要求

- soc平台无要求
- Android12以上
- zygisk开启并且版本v4以上(magisk v26.0以上并且开启zygisk / ksu + zygisk-next)

## 特殊说明

- 对开启fas的游戏使用shamiko会导致不生效
- 采用zygisk注入劫持libgui获取frametime，存在部分被检测风险
