# Mistpaper development

- This is a native Rust + GPUI writing client. Keep the release path free of JS runtimes, WebView and external development hosts.
- Preserve the centered 500 DIP column, 20 DIP Chinese body, 30 DIP line height and natural character spacing. Keep per-document font selection.
- Preserve GPUI Input's native scroll, selection and IME paths. Do not mirror scroll through app state or remeasure the whole document on every gesture event.
- Keep serialization and filesystem work off input handlers. Keep one pending save snapshot; snapshots share document bodies. Flush the newest snapshot on normal exit.
- Keep dependency revisions and Cargo.lock pinned. Updating the native text/render stack requires long Chinese text and real touchpad checks.
- Run cargo fmt --all --check, cargo test --release --locked and cargo clippy --release --locked --all-targets -- -D warnings. Build sequentially; cap CARGO_BUILD_JOBS on memory-constrained machines.
- Use MISTPAPER_DATA_DIR and the supplied scripts for isolated tests. Never commit user drafts, runtime logs, profiles, build outputs or credentials.
- Report source/build checks separately from actual UI, IME and physical touchpad verification. Do not describe save latency as input latency or FPS.
