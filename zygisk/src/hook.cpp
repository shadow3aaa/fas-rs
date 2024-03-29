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

#include <cstring>

#include "zygisk.hpp"

using zygisk::Api;
using zygisk::AppSpecializeArgs;
using zygisk::Option;
using zygisk::ServerSpecializeArgs;

#define LOGD(...) \
    __android_log_print(ANDROID_LOG_DEBUG, "libgui-zygisk", __VA_ARGS__)

class LibGuiHook : public zygisk::ModuleBase {
   public:
    void onLoad(Api *api, JNIEnv *env) override {
        api_ = api;
        env_ = env;
    }

    void postAppSpecialize(
        [[maybe_unused]] const AppSpecializeArgs *args) override {
        const char *process = env_->GetStringUTFChars(args->nice_name, nullptr);
        const bool need_hook = rust::need_hook(process);
        env_->ReleaseStringUTFChars(args->nice_name, process);

        if (need_hook) {
            rust::hook_handler();
        } else {
            api_->setOption(Option::DLCLOSE_MODULE_LIBRARY);
        }
    }

    void postServerSpecialize(
        [[maybe_unused]] const ServerSpecializeArgs *args) override {
        api_->setOption(Option::DLCLOSE_MODULE_LIBRARY);
    }

   private:
    Api *api_;
    JNIEnv *env_;
};

REGISTER_ZYGISK_MODULE(LibGuiHook)
