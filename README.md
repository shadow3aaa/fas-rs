# **FAS-RS**

- fas-rs程序在安卓平台运行
- fas-rs通过某种方式在监听帧变化，并且将此数据用于性能调度
- [todo-list](update/todo.md)
- [change-log](update/changelog.md)

## **配置合并**

- ### fas-rs内置配置合并系统，来解决未来的配置功能变动问题。它的行为如下

  - 剔除本地配置中，标准配置不存在的配置键值对
  - 插入本地配置缺少，标准配置存在的配置键值对
  - 保留标准配置和本地配置都存在的配置键的值

    **Note: 以上行为是未开启[keep_std](#keep_std)模式的行为(默认开启)，开启后合并行为详见[keep_std](#keep_std)**

- ### 注意

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
- true: 永远在配置合并时保持标准配置的profile，保留本地配置的应用列表，其它地方和false相同 *
- false: [配置合并的默认行为](#配置合并)

### **ignore_little**

- 类型: 布尔
- 可用值: true false
- true: 在机器至少有3个及以上的集簇时，fas-rs只控制非小核集簇
- false: fas-rs始终控制所有集群 *

#### **\* : 默认配置**

## **应用列表配置**

### **"package" = target_fps / "auto"**

- package: 字符串，应用包名
- target_fps / "auto": 任意正整数，或者字符串"auto"，表示锁定应用运行的目标fps，"auto"则是自动判断
  - "auto"是推荐选项，它在运行时判断目标fps，也就是说能够适配游戏不同的界面有不同的fps，然而，它只支持30/45/48/60/90/120/144这几种常见目标fps的判断，如果你的游戏不属于以上的任意一种，那么应该手动指定而不使用"auto"

## **powersave / balance / performance / fast mode配置**

- mode: 对应scene中的4模式
- jank_rec_count(u8): (big)jank发生后，多少个标准渲染时间后可开始降频
- normal_keep_count(u8): 降频发生后，下一次降频至少要经过多少个标准渲染时间
- tolerant_frame_jank(f64 / i32): 近5帧平均fps < 标准fps - tolerant_frame_jank 时判定为jank，进行升频
- tolerant_frame_limit(f64 / i32): 近5帧平均fps > 标准fps - tolerant_frame_limit 时判定为无jank，进行降频

### **示例**

```toml
[config]
ignore_little = false
keep_std = true

[game_list]
"com.hypergryph.arknights" = "auto"
"com.miHoYo.Yuanshen" = "auto"
"com.miHoYo.enterprise.NGHSoD" = "auto"
"com.miHoYo.hkrpg" = "auto"
"com.mojang.minecraftpe" = "auto"
"com.netease.party" = "auto"
"com.netease.wotb" = "auto"
"com.netease.x19" = "auto"
"com.pixeltoys.freeblade" = "auto"
"com.prpr.musedash.TapTap" = "auto"
"com.shangyoo.neon" = "auto"
"com.tencent.tmgp.pubgmhd" = "auto"
"com.tencent.tmgp.sgame" = "auto"

[powersave]
normal_keep_count = 2
jank_rec_count = 4
tolerant_frame_jank = 5.0
tolerant_frame_limit = 4.9

[balance]
normal_keep_count = 3
jank_rec_count = 6
tolerant_frame_jank = 3.0
tolerant_frame_limit = 2.9

[performance]
normal_keep_count = 3
jank_rec_count = 6
tolerant_frame_jank = 2.0
tolerant_frame_limit = 1.9

[fast]
normal_keep_count = 4
jank_rec_count = 8
tolerant_frame_jank = 1.0
tolerant_frame_limit = 1.0
```

## **编译(termux为例)**

```bash
# clone
git clone https://github.com/shadow3aaa/fas-rs

# install deps
apt install rust zip ndk* clang binutils-is-llvm xmake

# build & package
xmake
```

## **🐷🐷(🐷🐷🐷)**

- 🐷🐷🐷🐷🐷🐷
- 🐷🐷🐷🐷🐷🐷
- 🐷🐷🐷🐷🐷🐷
- 🐷🐷🐷🐷🐷🐷
- 🐷🐷🐷🐷🐷🐷
