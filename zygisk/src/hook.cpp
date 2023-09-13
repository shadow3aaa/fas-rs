/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
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
