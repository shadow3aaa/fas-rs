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
    fas-rs --merge --local-profile /path/to/local/config --std-profile /path/to/std/config
    ```

## **参数**

配置文件位于`/sdcard/Android/fas-rs/games.toml`

### **keep_std**

- 类型: 布尔
- 可用值: true false
- true: 永远在配置合并时保持标准配置的profile，保留本地配置的应用列表 *
- false: 详见[配置合并](#配置合并)

### **ignore_little**

- 类型: 布尔
- 可用值: true false
- true: 在机器至少有3个以上的核心机簇时，fas-rs只控制非小核集群 *
- false: fas-rs始终控制所有集群

### **min_step**

- 类型: 整数
- 可用值: 任意正整数
- 作用: Fas每次调整最大频率时的最小粒度，越大变化越大，单位是Mhz(频率)

### **powersave/balance/performance/fast + thermal**

- 类型: 整数
- 可用值: 任意正整数
- 作用: 对应模式触发温控限制的电池温度，默认为`balance`单位是`摄氏度 * 1000`，比如25.5摄氏度就是25500

#### **\* : 默认配置**

## **应用列表配置**

### **Package = target_fps**

- Package: 字符串，应用包名
- target_fps: 正整数，表示应用运行的目标fps

### **示例**

```toml
[config]
ignore_little = true
keep_std = true
min_step = 20
powersave_thermal = 36000
balance_thermal = 38000
performance_thermal = 42000
fast_thermal = 46000

[game_list]
"com.miHoYo.Yuanshen" = 60
"com.miHoYo.enterprise.NGHSoD" = 60
"com.miHoYo.hkrpg" = 60
"com.mojang.minecraftpe" = 120
"com.netease.x19" = 120
"com.pixeltoys.freeblade" = 60
"com.prpr.musedash.TapTap" = 60
"com.shangyoo.neon" = 60
"com.tencent.tmgp.pubgmhd" = 60
"com.tencent.tmgp.sgame" = 120
```

## 编译(termux为例)

```bash
# clone
git clone https://github.com/shadow3aaa/fas-rs
git submodule init
# Synchronize subprojects
git submodule update --init --recursive
# install deps
apt install zip ndk* clang binutils-is-llvm
# make debug
make
# make release
make RELEASE=true
```
