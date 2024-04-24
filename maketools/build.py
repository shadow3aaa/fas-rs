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
import shutil
from pathlib import Path
from maketools.toolchains import Buildtools
from maketools.misc import eprint

build_help_text = """\
python3 ./make.py build:
    --help:
        print this help
    --clean:
        clean up
    --check:
        run cargo check
    --release:
        release build
    --debug:
        debug build
    --nightly:
        Introducing more optimizations using rust nightly
    --verbose:
        print details of build\
"""
CFLAGS = (
    "-Ofast -flto -fmerge-all-constants -fno-exceptions -fomit-frame-pointer -fshort-enums \
-Wl,-O3,--lto-O3,--gc-sections,--as-needed,--icf=all,-z,norelro,--pack-dyn-relocs=android+relr \
-std=c++2b -Wall -lc++"
)


def __parse_args(args):
    check = False
    release = False
    debug = False
    build = False
    verbose = False
    clean = False
    nightly = False

    for arg in args:
        match arg:
            case "--release" | "-r":
                release = True
                build = True
            case "--debug" | "-d":
                debug = True
                build = True
            case "--clean":
                clean = True
            case "--nightly":
                nightly = True
            case "--verbose" | "verbose" | "-v":
                verbose = True
            case "--check":
                check = True
                build = False
            case "-h" | "--help":
                print(build_help_text)
            case _:
                raise Exception("Illegal build parameter: {}".format(arg))

    if not build and not clean and not check:
        raise Exception(
            "Missing necessary build task argument(--release / --debug / --clean)"
        )
    elif (release and debug) or (build and clean) or (check and clean):
        raise Exception("Conflicting build arguments")

    return (check, clean, release, nightly, verbose)


def __clean():
    try:
        shutil.rmtree("output")
    except Exception:
        pass

    os.system("cargo clean")
    os.chdir("zygisk")

    try:
        shutil.rmtree("output")
    except Exception:
        pass

    os.chdir("rust")
    os.system("cargo clean")


def __build_zygisk(
    tools: Buildtools, check: bool, release: bool, verbose: bool, nightly: bool
):
    root = Path.cwd()
    zygisk_root = root.joinpath("zygisk")
    os.chdir(zygisk_root)

    try:
        Path("output").mkdir()
    except Exception:
        pass

    os.chdir("rust")

    if nightly:
        cargo = tools.cargo_nightly()
    else:
        cargo = tools.cargo()

    if check:
        cargo.arg("check --target aarch64-linux-android")
    else:
        cargo.arg("build --target aarch64-linux-android")
        if nightly:
            cargo.arg("-Z build-std")

    if release:
        cargo.arg("--release")
    if verbose:
        cargo.arg("--verbose")

    cargo.build()

    if check:
        print("Finish check (zygisk)")
        return

    os.chdir(zygisk_root)

    source = Path("rust").joinpath("target").joinpath("aarch64-linux-android")

    if release:
        source = source.joinpath("release")
    else:
        source = source.joinpath("debug")

    source = source.joinpath("librust.a")
    destination = Path("output").joinpath("librust.a")
    shutil.copy2(source, destination)

    output = Path("output").joinpath("arm64-v8a.so")

    (
        tools.cpp()
        .arg("--shared {}".format(Path("src").joinpath("*.cpp")))
        .arg("-I {}".format(Path("rust").joinpath("include")))
        .arg("-L output -L {}".format(Path("..").joinpath("prebuilt")))
        .arg("-fPIC -nostdlib++ -Wl,-lrust,-llog,-lbinder_ndk")
        .arg(CFLAGS)
        .arg("-o {}".format(output))
        .arg("-Wl,--threads=1")
        .build()
    )

    tools.strip(output)
    os.chdir(root)


def __task_zygisk(args):
    try:
        tools = Buildtools()
    except Exception as err:
        eprint(err)
        exit(-1)

    try:
        (check, clean, release, nightly, verbose) = __parse_args(args)
    except Exception as err:
        eprint(err)
        exit(-1)

    if clean:
        __clean()
        return

    try:
        Path("output").mkdir()
    except Exception:
        pass

    if release:
        temp_dir = Path("output").joinpath(".temp").joinpath("release")
    else:
        temp_dir = Path("output").joinpath(".temp").joinpath("debug")

    try:
        shutil.rmtree(temp_dir)
    except Exception:
        pass

    __build_zygisk(tools, check, release, nightly, verbose)

    if nightly:
        cargo = tools.cargo_nightly()
    else:
        cargo = tools.cargo()

    if check:
        cargo.arg("check --target aarch64-linux-android")
    else:
        cargo.arg("build --target aarch64-linux-android")
        if nightly:
            cargo.arg("-Z build-std")

    if release:
        cargo.arg("--release")
    if verbose:
        cargo.arg("--verbose")

    cargo.rust_flag("-C default-linker-libraries")
    cargo.arg("--features use_binder")
    cargo.arg("--no-default-features")
    cargo.build()

    if check:
        print("Finish check")
        return

    module = Path("module").joinpath("fas-rs-zygisk")
    shutil.copytree(module, temp_dir)
    zygisk_lib = Path("zygisk").joinpath("output").joinpath("arm64-v8a.so")
    zygisk_module = temp_dir.joinpath("zygisk")
    zygisk_module.mkdir()
    shutil.copy2(zygisk_lib, zygisk_module)

    bin = Path("target").joinpath("aarch64-linux-android")
    if release:
        bin = bin.joinpath("release")
    else:
        bin = bin.joinpath("debug")
    bin = bin.joinpath("fas-rs")

    bin_module = temp_dir.joinpath("fas-rs")
    shutil.copy2(bin, bin_module)
    tools.strip(bin_module)

    if release:
        output = Path("output").joinpath("fas-rs-zygisk(release)")
    else:
        output = Path("output").joinpath("fas-rs-zygisk(debug)")
    shutil.make_archive(output, "zip", temp_dir)


def __task_ebpf(args):
    try:
        tools = Buildtools()
    except Exception as err:
        eprint(err)
        exit(-1)

    try:
        (check, clean, release, nightly, verbose) = __parse_args(args)
    except Exception as err:
        eprint(err)
        exit(-1)

    if clean:
        __clean()
        return

    try:
        Path("output").mkdir()
    except Exception:
        pass

    if release:
        temp_dir = Path("output").joinpath(".temp").joinpath("release")
    else:
        temp_dir = Path("output").joinpath(".temp").joinpath("debug")

    try:
        shutil.rmtree(temp_dir)
    except Exception:
        pass

    if nightly:
        cargo = tools.cargo_nightly()
    else:
        cargo = tools.cargo()

    if check:
        cargo.arg("check --target aarch64-linux-android")
    else:
        cargo.arg("build --target aarch64-linux-android")
        if nightly:
            cargo.arg("-Z build-std")

    if release:
        cargo.arg("--release")
    if verbose:
        cargo.arg("--verbose")

    cargo.rust_flag("-C default-linker-libraries")
    cargo.arg("--features use_ebpf")
    cargo.build()

    if check:
        print("Finish check")
        return

    module = Path("module").joinpath("fas-rs-ebpf")
    shutil.copytree(module, temp_dir)
    bin = Path("target").joinpath("aarch64-linux-android")
    if release:
        bin = bin.joinpath("release")
    else:
        bin = bin.joinpath("debug")
    bin = bin.joinpath("fas-rs")

    bin_module = temp_dir.joinpath("fas-rs")
    shutil.copy2(bin, bin_module)
    tools.strip(bin_module)

    if release:
        output = Path("output").joinpath("fas-rs-ebpf(release)")
    else:
        output = Path("output").joinpath("fas-rs-ebpf(debug)")
    shutil.make_archive(output, "zip", temp_dir)


def task(args):
    __task_zygisk(args)
    __task_ebpf(args)
