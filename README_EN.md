# **fas-rs**

[![English][readme-cn-badge]][readme-cn-url]
[![Stars][stars-badge]][stars-url]
[![CI Build][ci-badge]][ci-url]
[![Release][release-badge]][release-url]
[![Download][download-badge]][download-url]
[![Telegram][telegram-badge]][telegram-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: README.md
[stars-badge]: https://img.shields.io/github/stars/shadow3aaa/fas-rs?style=for-the-badge&logo=github
[stars-url]: https://github.com/shadow3aaa/fas-rs
[ci-badge]: https://img.shields.io/github/actions/workflow/status/shadow3aaa/fas-rs/ci.yml?style=for-the-badge&label=CI%20Build&logo=githubactions
[ci-url]: https://github.com/shadow3aaa/fas-rs/actions/workflows/ci.yml
[release-badge]: https://img.shields.io/github/v/release/shadow3aaa/fas-rs?style=for-the-badge&logo=rust
[release-url]: https://github.com/shadow3aaa/fas-rs/releases/latest
[download-badge]: https://img.shields.io/github/downloads/shadow3aaa/fas-rs/total?style=for-the-badge&logo=download
[download-url]: https://github.com/shadow3aaa/fas-rs/releases/latest
[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/fas_rs_official

## **Introduction**

> If the picture seen by the naked eye can be directly reflected in the scheduling, that is to say, the scheduler is placed from the perspective of the viewer to determine the performance, can perfect performance control and maximized experience be achieved? `FAS (Frame Aware Scheduling)` is this scheduling concept, trying to control performance by monitoring screen rendering to minimize overhead while ensuring rendering time.

- ### **What is `fas-rs`?**

  - `fas-rs` is an implementation of `FAS (Frame Aware Scheduling)` running in user mode. Compared with `MI FEAS` in kernel mode, it has the same core idea but has the advantages of almost universal compatibility and flexibility on any device.

## **Extension System**

- In order to maximize the flexibility of user mode, `fas-rs` has its own extension system. For development instructions, please see our [extension template repository](https://github.com/shadow3aaa/fas-rs-extension-module-template)

## **Customization (configuration)**

- ### **Configuration path: `/sdcard/Android/fas-rs/games.toml`**

- ### **Parameter (`config`) description:**

  - **keep_std**

    - Type: `bool`
    - `true`: Always keep the standard configuration profile when merging configurations, retain the local configuration application list, and other places are the same as false \*
    - `false`: see [default behavior of config merge](#config merge)

  - **scene_game_list**

    - Type: `bool`
    - `true`: Use scene game list
    - `false`: Not using scene game list

  - `*`: default configuration

- ### **Game list (`game_list`) description:**

  - **`"package"` = `target_fps`**

    - `package`: string, application package name
    - `target_fps`: an array (such as `[30, 60, 120, 144]`) or a single integer, indicating the target frame rate that the game will render to, `fas-rs` will dynamically match it at runtime

- ### **`powersave` / `balance` / `performance` / `fast` Description:**

  - **mode:**
    - Currently, `fas-rs` does not have an official switching mode manager, but is connected to the configuration interface of [`scene`](http://vtools.omarea.com). If you don’t use scene, the configuration of `balance` will be used by default.
    - If you have some understanding of programming on Linux, you can switch to the corresponding mode by writing any one of the 4 modes to the `/dev/fas_rs/mode` node, and at the same time, reading it can also know the current `fas-rs` mode
  - **Parameter Description:**
    - margin(ms): Allowed frame drop margin. The smaller the value, the higher the frame rate, the larger the value, the more power is saved (0 <= margin < 1000)

### **`games.toml` configuration standard example:**

```toml
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
margin = 3

[balance]
margin = 2

[performance]
margin = 1

[fast]
margin = 0
```

## **Configuration merge**

- ### `fas-rs` has a built-in configuration merging system to solve the problem of future configuration function changes. It behaves as follows

  - Delete configurations that do not exist in the local configuration and standard configuration
  - Insert the configuration where the local configuration is missing and the standard configuration exists
  - Retain configurations that exist in both standard and local configurations

- ### Notice

  - Implemented using automatic serialization and deserialization, unable to save non-serialization necessary information such as comments
  - The automatic merged configuration during installation will not be applied immediately, otherwise it may affect the operation of the current version. Instead, the local one will be replaced with the new merged configuration during the next restart.

- ### Manual merge

  - The module will be automatically called once every time it is installed.
  - Manual example

    ```bash
    fas-rs merge /path/to/std/profile
    ```

## **Compile**

```bash
# Ubuntu(NDK is required)
apt install gcc-multilib git-lfs clang python3

# ruff(python lints & format)
pip install ruff

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Cargo-ndk
cargo install cargo-ndk

# Clone
git clone https://github.com/shadow3aaa/fas-rs
cd fas-rs

# Compile
python3 ./make.py build --release
```
