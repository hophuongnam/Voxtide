fn main() {
    // Android: oboe/cpal pull in C++ symbols (e.g. `__cxa_pure_virtual`), but the
    // cdylib link doesn't pull the C++ runtime by default — leaving the symbol
    // undefined with no `DT_NEEDED` to resolve it, so `dlopen` of libvoxtide_lib.so
    // fails at startup (UnsatisfiedLinkError). Linking the shared STL adds a NEEDED
    // entry for libc++_shared.so (which must also be present in jniLibs at runtime).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo:rustc-link-lib=dylib=c++_shared");
    }
    tauri_build::build();
}
