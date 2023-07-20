WIP

# 配置
### 参数配置:
### EMA_TYPE
- 类型: 字符串
- 可用值: EMA DEMA None
- 作用: 指定为Cycles调速器获取的Cycles进行指数平滑处理的算法
- EMA: 指数移动平滑
- DEMA: 双重指数移动平滑，比EMA更加变化敏感
- None: 不进行指数平滑 *
### EMA_WIN
- 类型: 整数
- 可用值: 任意正整数
- 作用: 指定Cycles指数平滑窗口大小
- 建议: 越大越平滑，对变化越不敏感，越小越不平滑，对变化越敏感，不建议过大
- 4 *
### always_on_gov
- 类型: 布尔
- 可用值: true false
- true: 总是开启Cycles调速器，即使不处于Fas开启状态
- false: 只在Fas时使用Cycles调速器 *
### default_target_diff
- 类型: 整数
- 可用值: 任意正整数
- 作用: 作为always_on_gov开启时，非Fas状态下的Cycles调速器的余量，单位是Mhz
- 450 *
### default_target_diff_fas
- 类型: 整数
- 可用值: 任意正整数
- 作用: Fas启动时Cycles调速器的默认余量，单位是Mhz
- 600 *
### diff_move
- 类型: 整数
- 可用值: 任意正整数
- 作用: Fas每次调整目标余量时的大小，越大余量变化越大，单位是Mhz
- 75 *

*: 意味着这是默认配置
___
### Fas应用列表配置
包名 = \[目标Fps, 帧渲染时间窗口\]
* 包名: 字符串，应用包名
* 目标Fps: 整数，表示游戏理论上正常运行的Fps
* 帧渲染时间窗口: 整数，表示帧监视器分析历史帧时间的帧的个数，越大越稳定，但是太大了会导致过于保守的Fas

### 示例
```toml
[config]
EMA_TYPE = "None"
EMA_WIN = 4
always_on_gov = false
# 单位: Mhz，diff > 0Mhz
default_target_diff = 450
default_target_diff_fas = 600
diff_move = 75

[game_list]
# 和平精英
"com.tencent.tmgp.pubgmhd" = [60, 10]
# Freeblade(战锤40k: 自由之刃)
"com.pixeltoys.freeblade" = [60, 10]
# 喵斯快跑
"com.prpr.musedash.TapTap" = [60, 10]
# Minecraft(我的世界)
"com.mojang.minecraftpe" = [120, 10]
# 我的世界(网易答辩版)
"com.netease.x19" = [120, 10]
# 崩坏3
"com.miHoYo.enterprise.NGHSoD" = [60, 10]
# 原神
"com.miHoYo.Yuanshen" = [60, 10]
# 王者荣耀
"com.tencent.tmgp.sgame" = [120, 10]
```
