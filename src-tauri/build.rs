fn main() {
    // The key is baked in by `option_env!`, so a change must invalidate the build.
    println!("cargo:rerun-if-env-changed=POSTHOG_API_KEY");
    tauri_build::build()
}
