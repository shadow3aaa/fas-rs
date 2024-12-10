<div align="center">

<img src="assets/icon.svg" width="160" height="160" style="display: block; margin: 0 auto;" alt="SVG Image">

# **fas-rs**

### Frame aware scheduling for android

[![简体中文][readme-cn-badge]][readme-cn-url]
[![Stars][stars-badge]][stars-url]
[![CI Build][ci-badge]][ci-url]
[![Release][release-badge]][release-url]
[![Download][download-badge]][download-url]
[![Telegram][telegram-badge]][telegram-url]

</div>

> **⚠ Warning**: This document is gpt-translated and may contain inaccuracies or errors.

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

> If the scene seen by the naked eye can be directly reflected in the scheduling, that is, if the scheduler is placed from the viewer's perspective to decide performance, can perfect performance control and maximum experience be achieved? `FAS (Frame Aware Scheduling)` is this scheduling concept, which tries to control performance by monitoring frame rendering to minimize overhead while ensuring rendering time.

- ### **What is `fas-rs`?**

  - `fas-rs` is a user-space implementation of `FAS (Frame Aware Scheduling)`, which has the advantage of near-universal compatibility and flexibility on any device compared to the kernel-space `MI FEAS`.

## **Extension System**

- To maximize user-space flexibility, `fas-rs` has its own extension system. For development instructions, see the [extension template repository](https://github.com/shadow3aaa/fas-rs-extension-module-template).

## **Customization (Configuration)**

- ### **Configuration Path: `/sdcard/Android/fas-rs/games.toml`**

- ### **Parameter (`config`) Description:**

  - **keep_std**

    - Type: `bool`
    - `true`: Always keep the standard configuration profile when merging configurations, retaining the local configuration's application list, and other aspects are the same as false \*
    - `false`: See [default behavior of configuration merging](#configuration-merging)

  - **scene_game_list**

    - Type: `bool`
    - `true`: Use scene game list \*
    - `false`: Do not use scene game list

  - `*`: Default configuration

- ### **Game List (`game_list`) Description:**

  - **`"package"` = `target_fps`**

    - `package`: String, application package name
    - `target_fps`: An array (e.g., `[30, 60, 120, 144]`) or a single integer, representing the target frame rate the game will render to, `fas-rs` will dynamically match at runtime.

- ### **Modes (`powersave` / `balance` / `performance` / `fast`) Description:**

  - #### **Mode Switching:**

    - Currently, `fas-rs` does not have an official mode switching manager but integrates with the [`scene`](http://vtools.omarea.com) configuration interface. If you do not use scene, the default `balance` configuration is used.
    - If you have some understanding of programming on Linux, you can switch to the corresponding mode by writing any of the 4 modes to the `/dev/fas_rs/mode` node, and you can also read it to know the current mode of `fas-rs`.

  - #### **Mode Parameter Description:**

    - **margin:**

      - Type: `integer`
      - Unit: `milliseconds`
      - Allowed frame drop margin, the smaller the margin, the higher the frame rate, the larger the margin, the more power-saving (0 <= margin < 1000)

    - **core_temp_thresh:**

      - Type: `integer` or `"disabled"`
      - `integer`: Core temperature to trigger thermal control by `fas-rs` (unit 0.001℃)
      - `"disabled"`: Disable `fas-rs` built-in thermal control

### **Standard Example of `games.toml` Configuration:**

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
core_temp_thresh = 80000

[balance]
margin = 2
core_temp_thresh = 90000

[performance]
margin = 1
core_temp_thresh = 95000

[fast]
margin = 0
core_temp_thresh = 95000
```

## **Configuration Merging**

- ### `fas-rs` has a built-in configuration merging system to address future configuration feature changes. Its behavior is as follows

  - Delete configurations in the local configuration that do not exist in the standard configuration
  - Insert configurations that are missing in the local configuration but exist in the standard configuration
  - Retain configurations that exist in both the standard and local configurations

- ### Note

  - Implemented using automatic serialization and deserialization, unable to preserve comments and other non-serialization necessary information
  - The automatic merging configuration during installation will not be applied immediately to avoid affecting the current version's operation but will replace the local configuration with the merged new configuration on the next restart.

- ### Manual Merging

  - The module will automatically call once every time it is installed
  - Manual example

    ```bash
    fas-rs merge /path/to/std/profile
    ```

## **Compilation**

```bash
# Ubuntu (NDK is required)
apt install gcc-multilib git-lfs clang python3

# ruff (Python lints & format)
pip install ruff

# Rust (Nightly version is required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
rustup component add rust-src

# Cargo-ndk
cargo install cargo-ndk

# Clone
git clone https://github.com/shadow3aaa/fas-rs
cd fas-rs

# Compile
python3 ./make.py build --release
# Use the `--nightly` option when building(Some nightly flags will be added to produce smaller artifacts)
python3 ./make.py build --release --nightly
```
