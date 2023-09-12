/* Copyright 2022-2023 John "topjohnwu" Wu
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

#include <cstdlib>
#include <unistd.h>
#include <fcntl.h>
#include <string.h>
#include <android/log.h>
#include <rust.h>

#include "zygisk.hpp"

using zygisk::Api;
using zygisk::AppSpecializeArgs;
using zygisk::ServerSpecializeArgs;

#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, "libgui hook", __VA_ARGS__)

class LibGuiHook : public zygisk::ModuleBase {
public:
    void onLoad(Api *api, JNIEnv *env) override {
        this->api = api;
        this->env = env;
    }
 
    void postAppSpecialize(const AppSpecializeArgs *args) override {
        const uid_t uid = args->uid;
        const gid_t gid = args->gid;
        
        if (uid < 10000 || gid < 10000)
            return;
            
        const char *process = env->GetStringUTFChars(args->nice_name, nullptr);

        rust::hook_handler(process);

        env->ReleaseStringUTFChars(args->nice_name, process);
    }

private:
    Api *api;
    JNIEnv *env;
};

// Register our module class and the companion handler function
REGISTER_ZYGISK_MODULE(LibGuiHook)

/* static void companion_handler(int fd) {
    if (fd == -1)
        return;
        
    rust::companion_handler(fd);
} */

// REGISTER_ZYGISK_COMPANION(companion_handler)
