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
#include <sys/types.h>
#include <unistd.h>

#include <cstring>

#include "zygisk.hpp"

using zygisk::Api;
using zygisk::AppSpecializeArgs;
using zygisk::Option;

#define LOGD(...) \
    __android_log_print(ANDROID_LOG_DEBUG, "libgui-zygisk", __VA_ARGS__)

class LibGuiHook : public zygisk::ModuleBase {
   public:
    void onLoad(Api *api, JNIEnv *env) override {
        this->api_ = api;
        this->env_ = env;
        this->need_hook_ = false;
    }

    void preAppSpecialize(AppSpecializeArgs *args) override {
        const char *process = env_->GetStringUTFChars(args->nice_name, nullptr);

        uid_t uid = args->uid;
        gid_t gid = args->gid;

        if (uid <= 10000 || gid < 10000 ||
            strstr(process, "zygisk") != nullptr) {
            need_hook_ = false;
            env_->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        const int socket = api_->connectCompanion();

        if (socket == -1) {
            LOGD("Failed to get socket");
            return;
        }

        size_t len = strlen(process) + 1;

        if (write(socket, &len, sizeof(len)) == -1) {
            close(socket);
            env_->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        if (write(socket, process, len) == -1) {
            close(socket);
            env_->ReleaseStringUTFChars(args->nice_name, process);
            return;
        }

        if (read(socket, &need_hook_, sizeof(need_hook_)) == -1) {
            need_hook_ = false;
        }

        close(socket);
        env_->ReleaseStringUTFChars(args->nice_name, process);
    }

    void postAppSpecialize(const AppSpecializeArgs * /*args*/) override {
        if (need_hook_) {
            rust::hook_handler();
        } else {
            api_->setOption(Option::DLCLOSE_MODULE_LIBRARY);
        }
    }

   private:
    Api *api_;
    JNIEnv *env_;
    bool need_hook_;
};

// NOLINTBEGIN(misc-use-anonymous-namespace,-warnings-as-errors)
static void root_handler(int socket) {
    size_t len = 0;
    if (read(socket, &len, sizeof(len)) == -1) {
        return;
    }

    char process[len];
    memset(process, 0, sizeof(process));

    if (read(socket, &process, sizeof(process)) == -1) {
        return;
    }

    bool need_hook = rust::need_hook(process);
    write(socket, &need_hook, sizeof(need_hook));
}
// NOLINTEND(misc-use-anonymous-namespace,-warnings-as-errors)

REGISTER_ZYGISK_MODULE(LibGuiHook)
REGISTER_ZYGISK_COMPANION(root_handler)
