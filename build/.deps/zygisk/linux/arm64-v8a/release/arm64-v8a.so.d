{
    files = {
        "build/.objs/zygisk/linux/arm64-v8a/release/zygisk/src/hook.cpp.o"
    },
    values = {
        "/data/data/com.termux/files/usr/bin/g++",
        {
            "-shared",
            "-fPIC",
            "-Lzygisk/output",
            "-Lprebuilt",
            "-s",
            "-lc++",
            "-lrust",
            "-lbinder_ndk",
            "-llog",
            "-nostdlib++"
        }
    }
}