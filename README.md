<div align="center">

<img src="https://github.com/shadow3aaa/fas-rs/raw/refs/heads/master/assets/icon.svg" width="160" height="160" style="display: block; margin: 0 auto;" alt="SVG Image">

# **fas-rs**

### Frame aware scheduling for android

[![English][readme-en-badge]][readme-en-url]
[![Stars][stars-badge]][stars-url]
[![CI Build][ci-badge]][ci-url]
[![Release][release-badge]][release-url]
[![Download][download-badge]][download-url]
[![Telegram][telegram-badge]][telegram-url]

</div>

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: README_EN.md
[stars-badge]: https://img.shields.io/github/stars/Tools-cx-app/fas-rs-next?style=for-the-badge&logo=github
[stars-url]: https://github.com/Tools-cx-app/fas-rs-next
[ci-badge]: https://img.shields.io/github/actions/workflow/status/Tools-cx-app/fas-rs-next/ci.yml?style=for-the-badge&label=CI%20Build&logo=githubactions
[ci-url]: https://github.com/Tools-cx-app/fas-rs-next/actions/workflows/ci.yml
[release-badge]: https://img.shields.io/github/v/release/Tools-cx-app/fas-rs-next?style=for-the-badge&logo=rust
[release-url]: https://github.com/Tools-cx-app/fas-rs-next/releases/latest
[download-badge]: https://img.shields.io/github/downloads/Tools-cx-app/fas-rs-next/total?style=for-the-badge
[download-url]: https://github.com/Tools-cx-app/fas-rs-next/releases/latest
[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/fas_rs_next

## **简介**

> 假如肉眼看到的画面能直接反映在调度上，也就是说以把调度器放在观看者的角度来决定性能，是否就能实现完美的性能控制和最大化体验? `FAS (Frame Aware Scheduling)`就是这种调度概念，通过监视画面渲染来尽量控制性能以在保证渲染时间的同时实现最小化开销

- ### **什么是`fas-rs`?**

  - `fas-rs`是运行在用户态的`FAS(Frame Aware Scheduling)`实现，对比核心思路一致但是在内核态的`MI FEAS`有着近乎在任何设备通用的兼容性和灵活性方面的优势

## **插件系统**

- 为了最大化用户态的灵活性，`fas-rs-next`有自己的一套插件系统，开发说明详见[插件的模板仓库](https://github.com/shadow3aaa/fas-rs-extension-module-template)

## **自定义(配置)**

- ### **配置路径: `/sdcard/Android/fas-rs/games.toml`**

  - **logger_level**

    - 类型: `String`
    - `info`: 使用 info 级别logger \*
    - `debug`: 使用 debug 级别logger

- ### **参数(`config`)说明:**

  - **keep_std**

    - 类型: `bool`
    - `true`: 永远在配置合并时保持标准配置的 profile，保留本地配置的应用列表，其它地方和 false 相同 \*
    - `false`: 见[配置合并的默认行为](#配置合并)

  - **scene_game_list**

    - 类型: `bool`
    - `true`: 使用 scene 游戏列表 \*
    - `false`: 不使用 scene 游戏列表

- ### **游戏列表(`game_list`)说明:**

  - **`"package"` = `target_fps`**

    - `package`: 字符串，应用包名
    - `target_fps`: 一个数组(如`[30，60，120，144]`)或者单个整数，表示游戏会渲染到的目标帧率，`fas-rs`会在运行时动态匹配

- ### **模式(`powersave` / `balance` / `performance` / `fast`)说明:**

  - #### **模式切换:**

    - 目前`fas-rs-next`还没有官方的切换模式的管理器，而是接入了[`scene`](http://vtools.omarea.com)的配置接口，如果你不用 scene 则默认使用`balance`的配置
    - 如果你有在 linux 上编程的一些了解，向`/data/adb/fas_rs/node`节点写入 4 模式中的任意一个即可切换到对应模式，同时读取它也可以知道现在`fas-rs`所处的模式

  - #### **模式参数说明:**

    - **margin_fps:**
      - 支持两种格式:
       1. 完整格式：`margin_fps = { base = <float>, <target_fps margin override> = <float>(可多项) }`
       2. 简写：`margin_fps = <float>`，等效`margin_fps = { base = <float> }`
      - 解释: 以 fps 为单位的额外允许掉帧量，除非用`target_fps margin override`强制指定`margin_fps`值，否则会根据公式(`target_fps / 60 * base`)缩放

    - **core_temp_thresh:**

      - 类型: `整数`或者`"disabled"`
      - `整数`: 让`fas-rs-next`触发温控的核心温度(单位0.001℃)
      - `"disabled"`: 关闭`fas-rs-next`内置温控

### **`games.toml`配置标准例:**

```toml
logger_level = "info"

[config]
keep_std = true
scene_game_list = true

[game_list]
"com.hypergryph.arknights" = [30, 60]
"com.miHoYo.Yuanshen" = [30, 60]
"com.miHoYo.enterprise.NGHSoD" = [30, 60, 90]
"com.miHoYo.hkrpg" = [30, 60]
"com.kurogame.mingchao" = [24, 30, 45, 60]
"com.pwrd.hotta.laohu" = [25, 30, 45, 60, 90]
"com.mojang.minecraftpe" = [60, 90, 120]
"com.netease.party" = [30, 60]
"com.shangyoo.neon" = 60
"com.tencent.tmgp.pubgmhd" = [60, 90, 120]
"com.tencent.tmgp.sgame" = [30, 60, 90, 120]

[powersave]
margin_fps = 3.0
core_temp_thresh = 80000

[balance]
margin_fps = 1.0
core_temp_thresh = 90000

[performance]
margin_fps = 0.3
core_temp_thresh = 95000

[fast]
margin_fps = 0
core_temp_thresh = 95000
```

## **配置合并**

- ### `fas-rs-next`内置配置合并系统，来解决未来的配置功能变动问题。它的行为如下

  - 删除本地配置中，标准配置不存在的配置
  - 插入本地配置缺少，标准配置存在的配置
  - 保留标准配置和本地配置都存在的配置

- ### 注意

  - 使用自动序列化和反序列化实现，无法保存注释等非序列化必须信息
  - 安装时的自动合并配置不会马上应用，不然可能会影响现版本运行，而是会在下一次重启时用合并后的新配置替换掉本地的

- ### 手动合并

  - 模块每次安装都会自动调用一次
  - 手动例

    ```bash
    fas-rs-next merge /path/to/std/profile
    ```

## **编译**

```bash
# Ubuntu (NDK is required)
apt install gcc-multilib git-lfs

# Rust (Nightly version is required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
rustup component add rust-src

# Cargo-ndk
cargo install cargo-ndk

# Clone
git clone https://github.com/Tools-cx-app/fas-rs-next
cd fas-rs-next

# Compile
cargo xtask build -r
```

## **捐赠**

[🐷🐷的爱发电](https://afdian.com/a/shadow3qaq)，你的捐赠可以增加🐷🐷维护开发此项目的动力。

## 鸣谢

- [shadow3aaa](https://github.com/shadow3aaa/)
