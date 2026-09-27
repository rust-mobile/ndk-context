# Unreleased

- Store the `AndroidContext` in a `RwLock` instead of a `static mut`, so `android_context()` no longer races with `initialize_android_context()` and `release_android_context()`. Raises the MSRV to 1.63, where `RwLock::new` became `const`.

# 0.1.1 (2022-04-19)

- Add `release_android_context()` function to remove `AndroidContext` when activity is destroyed. (#263)

# 0.1.0 (2022-02-14)

- Initial release! 🎉
