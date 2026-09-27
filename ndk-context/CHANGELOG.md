# Unreleased

- Store the `AndroidContext` in a `RwLock` instead of a `static mut`, so `android_context()` no longer races with `initialize_android_context()` and `release_android_context()`. Raises the MSRV to 1.63, where `RwLock::new` became `const`.
- Add `try_android_context()`, `try_initialize_android_context()` and `try_release_android_context()`, which return instead of panicking. A second `initialize_android_context()` now keeps the first context.

# 0.1.1 (2022-04-19)

- Add `release_android_context()` function to remove `AndroidContext` when activity is destroyed. (#263)

# 0.1.0 (2022-02-14)

- Initial release! 🎉
