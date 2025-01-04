#!/bin/python3
# Copyright 2023-2025, shadow3 (@shadow3aaa)
#
# This file is part of fas-rs.
#
# fas-rs is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option)
# any later version.
#
# fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
# details.
#
# You should have received a copy of the GNU General Public License along
# with fas-rs. If not, see <https://www.gnu.org/licenses/>.

import os
import platform
from pathlib import Path
import shutil


def find_ndk_home():
    if (ndk_home := os.getenv("ANDROID_NDK_HOME")) is not None:
        return Path(ndk_home)
    elif (ndk_home := os.getenv("NDK_HOME")) is not None:
        return Path(ndk_home)
    elif (android_home := os.getenv("ANDROID_HOME")) is not None:
        ndks = Path(android_home).joinpath("sdk").joinpath("ndk")
        ndk_home = next(ndks.iterdir())

        if ndk_home.exists():
            return ndk_home
        else:
            raise FileNotFoundError("Failed to find ndk from ANDROID_HOME")
    elif (android_home := Path.home().joinpath("Android")).exists():
        ndks = Path(android_home).joinpath("sdk").joinpath("ndk")
        ndk_home = next(ndks.iterdir())

        if ndk_home.exists():
            return ndk_home
        else:
            raise FileNotFoundError("Failed to find ndk from ANDROID_HOME")


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

        print("Rust build:")
        print("Working dir: {}".format(Path.cwd()))
        print("Command: {}".format(command))

        if os.system(command) != 0:
            raise Exception("Rust build failed!")


class CargoNightly:
    __cargo = ""
    __args = ""
    __extra_args = ""
    __rust_flags = ""

    def __init__(self):
        if os.getenv("TERMUX_VERSION") is not None:
            prefix = os.getenv("PREFIX")
            if prefix is not None:
                self.__cargo = Path(prefix).joinpath("opt/rust-nightly/bin/cargo")
            else:
                raise Exception("missing env 'PREFIX'")
        elif shutil.which("cargo-ndk") is not None:
            self.__cargo = "cargo +nightly ndk -p 31 -t arm64-v8a"
        else:
            raise Exception("Install cargo-ndk first")

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
        print("Rust build:")
        print("Working dir: {}".format(Path.cwd()))
        print("Command: {}".format(command))

        if os.system(command) != 0:
            raise Exception("Rust build failed!")


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
        print("C++ build:")
        print("Working dir: {}".format(Path.cwd()))
        print("Command: {}".format(command))

        if os.system(command) != 0:
            raise Exception("C++ build failed!")


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
        print("Clang tidy:")
        print("Working dir: {}".format(Path.cwd()))
        print("Command: {}".format(command))

        if os.system(command) != 0:
            raise Exception("Clang tidy failed!")


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
        else:
            ndk_home = find_ndk_home()
            system = platform.system()
            arch = platform.machine()
            if ndk_home is None:
                raise Exception("Failed to find ndk")
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
                case (_, "Darwin"):
                    _dir = prebuilt.joinpath("darwin-x86_64")
                    bins = _dir.joinpath("bin")
                    sysroot = _dir.joinpath("sysroot")
                    os.environ["BINDGEN_EXTRA_CLANG_ARGS"] = "--sysroot={}".format(
                        sysroot
                    )
                case _:
                    raise Exception("Unsupported platform: {} {}".format(arch, system))

            self.__cargo = "cargo ndk -p 31 -t arm64-v8a"
            self.__strip = bins.joinpath("llvm-strip")
            self.__clang_plusplus = bins.joinpath("aarch64-linux-android31-clang++")
            self.__clang_format = "clang-format"
            self.__clang_tidy = bins.joinpath("clang-tidy")

    def cargo(self):
        return Cargo(self.__cargo)

    def cargo_nightly(self):
        return CargoNightly()

    def strip(self, path: Path):
        command = "{} {}".format(self.__strip, path)
        os.system(command)

    def cpp(self):
        return Cpp(str(self.__clang_plusplus))

    def cpp_format(self, path: Path):
        command = "{} -i --verbose {}".format(self.__clang_format, path)

        if os.system(command) != 0:
            raise Exception("C++ codes format failed!")

    def cpp_tidy(self):
        return CppTidy(str(self.__clang_tidy))
