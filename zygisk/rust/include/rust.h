#pragma once

namespace rust {
    extern "C" {
        void hook_handler(const char* process);
    }
}