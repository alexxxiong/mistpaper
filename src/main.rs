#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod model;
mod store;

use nsh_platform_gpui as platform;

fn main() -> std::process::ExitCode {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn,nsh::fps=info"),
    )
    .init();
    let data = data_directory();
    if let Err(error) = store::initialize(&data) {
        log::error!("opening {}: {error}", data.display());
        return std::process::ExitCode::FAILURE;
    }
    let assets = std::env::current_exe()
        .expect("executable path")
        .parent()
        .expect("executable directory")
        .join("assets");
    let assets = if assets.is_dir() {
        assets
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
    };
    let exit = platform::run_app(
        platform::AppSource::Rust(
            platform::Root::of::<app::App>().app_id("studio.mistpaper.desktop"),
        ),
        platform::AppOptions {
            asset_base: Some(platform::AssetBase::Directory(assets)),
            appearance: nsh_core::prelude::Appearance::Light,
            title: "雾笺 · Mistpaper".into(),
            app_id: "studio.mistpaper.desktop".into(),
            size: (1152, 768),
            fps: std::env::var_os("MISTPAPER_FPS").is_some(),
            data_dir: Some(data.join("runtime")),
            ..Default::default()
        },
    );
    store::store().close();
    exit
}

fn data_directory() -> std::path::PathBuf {
    use std::{env, path::PathBuf};
    if let Some(dir) = env::var_os("MISTPAPER_DATA_DIR") {
        return dir.into();
    }
    #[cfg(windows)]
    let root = PathBuf::from(env::var_os("APPDATA").expect("APPDATA"));
    #[cfg(target_os = "macos")]
    let root =
        PathBuf::from(env::var_os("HOME").expect("HOME")).join("Library/Application Support");
    #[cfg(not(any(windows, target_os = "macos")))]
    let root = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").expect("HOME")).join(".local/share"));
    root.join("Mistpaper")
}
