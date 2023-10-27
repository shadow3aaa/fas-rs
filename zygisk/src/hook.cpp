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
#include <android/log.h>
#include <jni.h>
#include <rust.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#include "zygisk.hpp"

using zygisk::Api;
using zygisk::AppSpecializeArgs;
using zygisk::Option;

#define LOGD(...) \
    __android_log_print(ANDROID_LOG_DEBUG, "libgui-zygisk", __VA_ARGS__)

class LibGuiHook : public zygisk::ModuleBase {
   public:
    void onLoad(Api *api, JNIEnv *env) override {
        this->api = api;
        this->env = env;
        this->need_hook = false;
    }

    void preAppSpecialize(AppSpecializeArgs *args) override {
        bool hook = false;
        const int fd = api->connectCompanion();

        if (fd == -1) {
            LOGD("Failed to get socket");
            return;
        }

        LOGD("Root process connected");

        const char *process = env->GetStringUTFChars(args->nice_name, nullptr);

        LOGD("Sending process length to root process");
        size_t len = strlen(process) + 1;
        write(fd, &len, sizeof(len));

        LOGD("Sending process name to root process");
        write(fd, process, len);

        env->ReleaseStringUTFChars(args->nice_name, process);

        LOGD("Reading need_hook from root process");
        read(fd, &hook, sizeof(hook));
        close(fd);

        this->need_hook = hook;

        LOGD("preAppSpecialize ends");
    }

    void postAppSpecialize(const AppSpecializeArgs *args) override {
        if (need_hook) {
            const char *process =
                env->GetStringUTFChars(args->nice_name, nullptr);
            rust::hook_handler(process);
            env->ReleaseStringUTFChars(args->nice_name, process);
        } else {
            api->setOption(Option::DLCLOSE_MODULE_LIBRARY);
        }
    }

   private:
    Api *api;
    JNIEnv *env;
    bool need_hook;
};

static void companion_handler(int fd) {
    size_t len = 0;
    bool need_hook = false;

    LOGD("Reading process length from socket");
    read(fd, &len, sizeof(len));

    char process[len];
    memset(process, 0, sizeof(process));

    LOGD("Reading process from socket");
    read(fd, &process, sizeof(process));

    need_hook = rust::need_hook(process);

    LOGD("Writing need_hook: %d to socket", need_hook);
    write(fd, &need_hook, sizeof(need_hook));

    LOGD("Root process ends");
}

REGISTER_ZYGISK_MODULE(LibGuiHook)
REGISTER_ZYGISK_COMPANION(companion_handler)
