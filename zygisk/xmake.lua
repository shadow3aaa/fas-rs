set_languages("c++2b")
add_rules("mode.release", "mode.debug")

target("zygisk")
    set_kind("shared")
    set_filename("arm64-v8a.so")
    set_targetdir("output")

add_shflags("-nostdlib++")
add_links("rust", "binder_ndk", "log", "c++")
add_linkdirs("output", "../prebuilt")
add_files("src/*.cpp")
add_includedirs("rust/include")
