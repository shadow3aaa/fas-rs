#ifndef RUST_H
#define RUST_H

#pragma once

extern "C" {
void _hook_handler_(const char*);
auto _need_hook_(const char*) -> bool;
}

namespace rust {
inline void hook_handler(const char *process) {
   _hook_handler_(process);
}

inline auto need_hook(const char *process) -> bool { return _need_hook_(process); }
}  // namespace rust

#endif
