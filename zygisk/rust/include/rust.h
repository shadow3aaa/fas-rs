#pragma once

extern "C" bool __hook_handler__(const char*);

namespace rust {
inline bool hook_handler(const char* process_name) {
    return __hook_handler__(process_name);
}
}  // namespace rust
