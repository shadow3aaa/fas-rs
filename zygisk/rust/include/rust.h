#pragma once

namespace rust {
extern "C" {
void hook_handler();
auto need_hook(const char *process) -> bool;
}
}  // namespace rust
