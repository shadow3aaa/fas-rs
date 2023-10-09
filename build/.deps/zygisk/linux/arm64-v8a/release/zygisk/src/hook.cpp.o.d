{
    files = {
        "zygisk/src/hook.cpp"
    },
    depfiles_gcc = "build/.objs/zygisk/linux/arm64-v8a/release/zygisk/src/__cpp_hook.cpp.cpp:   zygisk/src/hook.cpp zygisk/rust/include/rust.h zygisk/src/zygisk.hpp\
",
    values = {
        "/data/data/com.termux/files/usr/bin/gcc",
        {
            "-Qunused-arguments",
            "-fPIC",
            "-O3",
            "-std=c++2b",
            "-Izygisk/rust/include",
            "-DNDEBUG"
        }
    }
}