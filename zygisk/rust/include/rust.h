#pragma once

namespace rust {
    extern "C" {
        // void companion_handler(int fd);
        void hook_handler(const char* process);
    }
}