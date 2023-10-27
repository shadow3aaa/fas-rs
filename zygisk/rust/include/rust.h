#ifndef RUST_H
#define RUST_H

#pragma once

extern "C" {
auto _hook_handler_(const char*) -> bool;
auto _need_hook_(const char*) -> bool;
}

namespace rust {
inline auto hook_handler(const char *process) -> bool {
    return _hook_handler_(process);
}

inline auto need_hook(const char *process) -> bool { return _need_hook_(process); }
}  // namespace rust

#endif
