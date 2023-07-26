# **FAS-RS**

- fas-rs程序在安卓平台运行
- fas-rs通过某种方式在监听帧变化，并且将此数据用于性能调度

## **配置合并**

- ### fas-rs内置配置合并系统，来解决未来的配置功能变动问题。它的行为如下

  - 剔除本地配置中，标准配置不存在的配置键值对
  - 插入本地配置缺少，标准配置存在的配置键值对
  - 保留标准配置和本地配置都存在的配置键的值

    **Note: 以上行为是未开启[keep_std](#keep_std)模式的行为(默认开启)，开启后合并行为详见[keep_std](#keep_std)**

- ### 已知缺陷

  - 使用自动序列化和反序列化实现，**无法保存注释**等非序列化必须信息

- ### 调用

  - 模块每次安装都会自动调用一次
  - 手动调用

    ```bash
    fas-rs merge /path/to/local/config /path/to/std/config
    ```

___

## **参数**

配置文件位于/sdcard/Android/fas-rs/games.toml

### **keep_std**

- 类型: 布尔
- 可用值: true false
- true: 永远在配置合并时保持标准配置的profile，保留本地配置的应用列表 *
- false: 详见[配置合并](#配置合并)

### **EMA_TYPE**

- 类型: 字符串
- 可用值: EMA DEMA SMA None
- 作用: 指定为Cycles调速器获取的Cycles进行指数平滑处理的算法
- EMA: 指数移动平滑
- DEMA: 双重指数移动平滑，比EMA更加变化敏感
- SMA: 简单均值平滑 *
- None: 不进行指数平滑

### **EMA_WIN**

- 类型: 整数
- 可用值: 任意正整数
- 作用: 指定Cycles指数平滑窗口大小
- 建议: 越大越平滑，对变化越不敏感，越小越不平滑，对变化越敏感，不建议过大
- 4 *

### **ignore_little**

- 类型: 布尔
- 可用值: true false
- true: Cycles调速器只调度后两个集群 *
- false: Cycles调速器调度所有集群

### **always_on_gov**

- 类型: 布尔
- 可用值: true false
- true: 总是开启Cycles调速器，即使不处于Fas开启状态
- false: 只在Fas时使用Cycles调速器 *

### **touch_boost**

- 类型: 整数
- 可用值: 任意正整数
- 作用: 触摸屏幕时提高频率，优先级小于[slide_boost](#slide_boost)
- 1 *

### **slide_boost**

- 类型: 整数
- 可用值: 任意正整数
- 作用: 触摸屏幕时提高频率，优先级大于[touch_boost](#touch_boost)
- 2 *

### **slide_timer**

- 类型: 整数
- 可用值: 任意正整数
- 作用: 停止滑动时保持[slide_boost](#slide_boost)状态的时间长度，单位ms
- 200 *

### **default_target_diff**

- 类型: 整数
- 可用值: 任意正整数
- 作用: 作为[always_on_gov](#always_on_gov)开启时，非Fas状态下的Cycles调速器的余量，单位是Mhz
- 450 *

### **default_target_diff_fas**

- 类型: 整数
- 可用值: 任意正整数
- 作用: Fas启动时Cycles调速器的默认余量，单位是Mhz
- 600 *

### **diff_move**

- 类型: 整数
- 可用值: 任意正整数
- 作用: Fas每次调整目标余量时的大小，越大余量变化越大，单位是Mhz
- 75 *

#### **\* : 默认配置**

___

## **应用列表配置**

### **Package = \[target_fps, frame_widow_len\]**

- Package: 字符串，应用包名
- target_fps: 正整数，表示应用运行的目标fps
- frame_widow_len: 整数，表示帧监视器分析历史帧时间的帧的窗口大小，越大越保守

### 示例

```toml
[config]
keep_std = true
EMA_TYPE = "SMA"
EMA_WIN = 4
always_on_gov = false
default_target_diff = 450
default_target_diff_fas = 600
ignore_little = true
diff_move = 75
slide_boost = 2
touch_boost = 1
slide_timer = 200

[game_list]
"com.miHoYo.Yuanshen" = [60, 10]
"com.miHoYo.enterprise.NGHSoD" = [60, 10]
"com.miHoYo.hkrpg" = [60, 10]
"com.mojang.minecraftpe" = [120, 10]
"com.netease.x19" = [120, 10]
"com.pixeltoys.freeblade" = [60, 10]
"com.prpr.musedash.TapTap" = [60, 10]
"com.tencent.tmgp.pubgmhd" = [60, 10]
"com.tencent.tmgp.sgame" = [120, 10]
```
