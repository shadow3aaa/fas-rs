add_rules("mode.release", "mode.debug")
includes("zygisk")

target("fas-rs")
    on_clean(function (target)
        os.exec("cargo clean")
        os.rmdir("output")
    end)
    
    before_build(function (target)
        os.rm("output/.temp")
        os.cp("module", "output/.temp")
    end)

    if is_mode("debug") then
        on_build(function (target)
            os.exec("cargo build --target aarch64-linux-android")
        end)
    else
        on_build(function (target)
            os.exec("cargo build --release --target aarch64-linux-android")
        end)
    end

target("package")
    add_deps("fas-rs")
    add_deps("zygisk")

    on_build(function (target)
        local temp = path.join(target:scriptdir(), "output/.temp")
        
        if is_mode("debug") then
            os.cp("target/aarch64-linux-android/debug/fas-rs", "output/.temp/fas-rs")
        else
            os.cp("target/aarch64-linux-android/release/fas-rs", "output/.temp/fas-rs")
        end
        
        os.cp("zygisk/output/arm64-v8a.so", "output/.temp/zygisk/arm64-v8a.so")
        
        os.cd(temp)
        os.exec("zip -9 -rq ../fas-rs.zip .")
        print("Flashable Module Packaged: output/fas-rs.zip")
    end)
