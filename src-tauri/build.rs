fn main() {
    // The key is baked in by `option_env!`, so a change must invalidate the build.
    println!("cargo:rerun-if-env-changed=POSTHOG_API_KEY");
    println!("cargo:rustc-check-cfg=cfg(posthog_api_key)");
    if std::env::var("POSTHOG_API_KEY").is_ok_and(|key| !key.is_empty()) {
        println!("cargo:rustc-cfg=posthog_api_key");
    }
    tauri_build::build()
}
