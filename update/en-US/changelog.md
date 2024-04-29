# v2.8.0

## Change log

- **Added `Ebpf & Uprobe` method instead of `Zygisk + Hook` to track libgui calls**

  - Starting from v2.8.0, both `Epbf version` and `Zygisk version` will be provided, and users need to choose by themselves.
  - Users with better kernel support for Ebpf are recommended to use the `Ebpf version` to achieve more efficiency and avoid hook detection.
  - Users with poor/unsupported kernel support for Ebpf (can only) use the `Zygisk version` to ensure compatibility

- Adjust calculation base frequency
- You can choose whether to read the scene game list
- Only force close `feas/migt` when fas-rs is loaded into the game
- Stable extension API to ensure backward compatibility
- Streamlined operation logic
- Optimize README visual effects
- Update dependencies
