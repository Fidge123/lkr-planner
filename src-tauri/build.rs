fn main() {
    // `option_env!("POSTHOG_API_KEY")` is baked in at compile time, so a changed
    // key must invalidate the cached build.
    println!("cargo:rerun-if-env-changed=POSTHOG_API_KEY");
    tauri_build::build()
}
