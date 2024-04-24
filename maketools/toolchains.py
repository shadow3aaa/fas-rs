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


class Cargo:
    __cargo = ""
    __args = ""
    __extra_args = ""
    __rust_flags = ""

    def __init__(self, command: str):
        self.__cargo = command

    def arg(self, arg: str):
        self.__args += "{} ".format(arg)
        return self

    def extra_arg(self, arg: str):
        self.__extra_args += "{} ".format(arg)
        return self

    def rust_flag(self, arg: str):
        self.__rust_flags += "{} ".format(arg)
        return self

    def build(self):
        command = "RUSTFLAGS='{}' {} {} -- {}".format(
            self.__rust_flags, self.__cargo, self.__args, self.__extra_args
        )
        os.system(command)


class CargoNightly:
    __cargo = ""
    __args = ""
    __extra_args = ""
    __rust_flags = ""

    def __init__(self):
        if os.getenv("TERMUX_VERSION") is not None:
            prefix = os.getenv("PREFIX")
            self.__cargo = Path(prefix).joinpath("opt/rust-nightly/bin/cargo")
        elif os.getenv("ANDROID_NDK_HOME") is not None:
            self.__cargo = "cargo +nightly ndk -p 31 -t arm64-v8a"
        else:
            raise Exception("Missing env: ANDROID_NDK_HOME")

    def arg(self, arg: str):
        self.__args += "{} ".format(arg)
        return self

    def extra_arg(self, arg: str):
        self.__extra_args += "{} ".format(arg)
        return self

    def rust_flag(self, arg: str):
        self.__rust_flags += "{} ".format(arg)
        return self

    def build(self):
        command = "RUSTFLAGS='{}' {} {} -- {}".format(
            self.__rust_flags, self.__cargo, self.__args, self.__extra_args
        )
        os.system(command)


class Cpp:
    __clang_plusplus = ""
    __args = ""

    def __init__(self, command: str):
        self.__clang_plusplus = command

    def arg(self, arg):
        self.__args += "{} ".format(arg)
        return self

    def build(self):
        command = "{} {}".format(self.__clang_plusplus, self.__args)
        os.system(command)


class CppTidy:
    __clang_tidy = ""
    __args = ""

    def __init__(self, command: str):
        self.__clang_tidy = command

    def arg(self, arg: str):
        self.__args += "{} ".format(arg)
        return self

    def tidy(self):
        command = "{} {}".format(self.__clang_tidy, self.__args)
        os.system(command)


class Buildtools:
    __cargo = ""
    __strip = ""
    __clang_plusplus = ""
    __clang_format = ""
    __clang_tidy = ""

    def __init__(self):
        if os.getenv("TERMUX_VERSION") is not None:
            self.__cargo = "cargo"
            self.__strip = "strip"
            self.__clang_plusplus = "clang++"
            self.__clang_format = "clang-format"
            self.__clang_tidy = "clang-tidy"
        elif (ndk_home := os.getenv("ANDROID_NDK_HOME")) is not None:
            system = platform.system()
            arch = platform.machine()
            prebuilt = (
                Path(ndk_home)
                .joinpath("toolchains")
                .joinpath("llvm")
                .joinpath("prebuilt")
            )

            match (arch, system):
                case ("x86_64", "Windows") | ("AMD64", "Windows"):
                    bins = prebuilt.joinpath("windows-x86_64").joinpath("bin")
                case ("x86_64", "Linux") | ("AMD64", "Linux"):
                    bins = prebuilt.joinpath("linux-x86_64").joinpath("bin")
                case ("aarch64", "Linux"):
                    bins = prebuilt.joinpath("linux-aarch64").joinpath("bin")
                case _:
                    raise Exception("Unsupported platform: {} {}".format(arch, system))

            self.__cargo = "cargo ndk -p 31 -t arm64-v8a"
            self.__strip = bins.joinpath("llvm-strip")
            self.__clang_plusplus = bins.joinpath("aarch64-linux-android31-clang++")
            self.__clang_format = "clang-format"
            self.__clang_tidy = bins.joinpath("clang-tidy")
        else:
            raise Exception("Missing env: ANDROID_NDK_HOME")

    def cargo(self):
        return Cargo(self.__cargo)

    def cargo_nightly(self):
        return CargoNightly()

    def strip(self, path: Path):
        command = "{} {}".format(self.__strip, path)
        os.system(command)

    def cpp(self):
        return Cpp(self.__clang_plusplus)

    def cpp_format(self, path: Path):
        command = "{} -i --verbose {}".format(self.__clang_format, path)
        os.system(command)

    def cpp_tidy(self):
        return CppTidy(self.__clang_tidy)
