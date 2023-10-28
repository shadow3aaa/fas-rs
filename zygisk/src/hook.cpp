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
        const char *process = env->GetStringUTFChars(args->nice_name, nullptr);

        uid_t uid = args->uid;
        gid_t gid = args->gid;

        if (uid <= 10000 || gid < 10000 ||
            strstr(process, "zygisk") != nullptr) {
            this->need_hook = false;
            env->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        const int fd = api->connectCompanion();

        if (fd == -1) {
            LOGD("Failed to get socket");
            return;
        }

        size_t len = strlen(process) + 1;

        if (write(fd, &len, sizeof(len)) == -1) {
            close(fd);
            env->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        if (write(fd, process, len) == -1) {
            close(fd);
            env->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        if (read(fd, &hook, sizeof(hook)) == -1) {
            close(fd);
            env->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        close(fd);
        env->ReleaseStringUTFChars(args->nice_name, process);

        this->need_hook = hook;
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

    if (read(fd, &len, sizeof(len)) == -1) {
        return;
    }

    char process[len];
    memset(process, 0, sizeof(process));

    if (read(fd, &process, sizeof(process)) == -1) {
        return;
    }

    need_hook = rust::need_hook(process);
    write(fd, &need_hook, sizeof(need_hook));
}

REGISTER_ZYGISK_MODULE(LibGuiHook)
REGISTER_ZYGISK_COMPANION(companion_handler)
