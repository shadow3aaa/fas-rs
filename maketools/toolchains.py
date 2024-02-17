#!/bin/python3
#
# Copyright 2023 shadow3aaa@gitbub.com
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
import os
import platform
from pathlib import Path
from maketools.misc import eprint

if os.getenv("TERMUX_VERSION") is not None:
    __cargo = "cargo"
    __strip = "strip"
    __clang_plusplus = "clang++"
    __clang_format = "clang-format"
    __clang_tidy = "clang-tidy"
elif (__ndk_home := os.getenv("ANDROID_NDK_HOME")) is not None:
    system = platform.system()
    arch = platform.machine()
    match (arch, system):
        case ("x86_64", "Windows") | ("AMD64", "Windows"):
            __bins = (
                Path(__ndk_home)
                .joinpath("toolchains")
                .joinpath("llvm")
                .joinpath("prebuilt")
                .joinpath("windows-x86_64")
                .joinpath("bin")
            )
        case ("x86_64", "Linux") | ("AMD64", "Linux"):
            __bins = (
                Path(__ndk_home)
                .joinpath("toolchains")
                .joinpath("llvm")
                .joinpath("prebuilt")
                .joinpath("linux-x86_64")
                .joinpath("bin")
            )
        case _:
            eprint("Unsupported platform: {} {}".format(arch, system))
            exit(-1)

    __cargo = "cargo ndk -p 31 -t arm64-v8a"
    __strip = __bins.joinpath("llvm-strip")
    __clang_plusplus = __bins.joinpath("aarch64-linux-android31-clang++")
    __clang_format = __bins.joinpath("clang-format")
    __clang_tidy = __bins.joinpath("clang-tidy")
else:
    eprint("Missing env: ANDROID_NDK_HOME")
    exit(-1)


def cargo(arg: str):
    os.system("{} {}".format(__cargo, arg))


def strip(arg: str):
    os.system("{} {}".format(__strip, arg))


def clang_plusplus(arg: str):
    os.system("{} {}".format(__clang_plusplus, arg))


def clang_format(arg: str):
    os.system("{} {}".format(__clang_format, arg))


def clang_tidy(arg: str):
    os.system("{} {}".format(__clang_tidy, arg))
