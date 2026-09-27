//! Provides a stable api to rust crates for interfacing with the Android platform. It is
//! initialized by the runtime, usually [__ndk-glue__](https://crates.io/crates/ndk-glue),
//! but could also be initialized by Java or Kotlin code when embedding in an existing Android
//! project.
//!
//! ```no_run
//! # use jni::objects::JObject;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let ctx = ndk_context::android_context();
//! let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
//! let context = unsafe { JObject::from_raw(ctx.context().cast()) };
//! let env = vm.attach_current_thread()?;
//! let class_ctx = env.find_class("android/content/Context")?;
//! let audio_service = env.get_static_field(class_ctx, "AUDIO_SERVICE", "Ljava/lang/String;")?;
//! let audio_manager = env
//!     .call_method(
//!         context,
//!         "getSystemService",
//!         "(Ljava/lang/String;)Ljava/lang/Object;",
//!         &[audio_service],
//!     )?
//!     .l()?;
//! # Ok(())
//! # }
//! ```
use std::{
    ffi::c_void,
    sync::{PoisonError, RwLock},
};

static ANDROID_CONTEXT: RwLock<Option<SharedContext>> = RwLock::new(None);

#[derive(Clone, Copy)]
struct SharedContext(AndroidContext);

// SAFETY: this crate only copies the addresses and never dereferences them, which is sound on any
// thread. Callers of `android_context` dereference them under their own `unsafe`.
unsafe impl Send for SharedContext {}
// SAFETY: a shared `SharedContext` only hands out copies of the addresses, as for `Send`.
unsafe impl Sync for SharedContext {}

/// [`AndroidContext`] provides the pointers required to interface with the jni on Android
/// platforms.
#[derive(Clone, Copy, Debug)]
pub struct AndroidContext {
    java_vm: *mut c_void,
    context_jobject: *mut c_void,
}

impl AndroidContext {
    /// A handle to the `JavaVM` object.
    ///
    /// Usage with [__jni__](https://crates.io/crates/jni) crate:
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let ctx = ndk_context::android_context();
    /// let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    /// let env = vm.attach_current_thread();
    /// # Ok(())
    /// # }
    /// ```
    pub fn vm(self) -> *mut c_void {
        self.java_vm
    }

    /// A handle to an [android.content.Context](https://developer.android.com/reference/android/content/Context).
    /// In most cases this will be a ptr to an `Activity`, but this isn't guaranteed.
    ///
    /// Usage with [__jni__](https://crates.io/crates/jni) crate:
    /// ```no_run
    /// # use jni::objects::JObject;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let ctx = ndk_context::android_context();
    /// let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    /// let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    /// let env = vm.attach_current_thread()?;
    /// let class_ctx = env.find_class("android/content/Context")?;
    /// let audio_service = env.get_static_field(class_ctx, "AUDIO_SERVICE", "Ljava/lang/String;")?;
    /// let audio_manager = env
    ///     .call_method(
    ///         context,
    ///         "getSystemService",
    ///         "(Ljava/lang/String;)Ljava/lang/Object;",
    ///         &[audio_service],
    ///     )?
    ///     .l()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn context(self) -> *mut c_void {
        self.context_jobject
    }
}

/// Main entry point to this crate. Returns an [`AndroidContext`].
///
/// # Panics
///
/// Panics if the context is not initialized.
pub fn android_context() -> AndroidContext {
    try_android_context().expect("android context was not initialized")
}

/// Returns the [`AndroidContext`] if it is initialized.
pub fn try_android_context() -> Option<AndroidContext> {
    let context = *ANDROID_CONTEXT
        .read()
        .unwrap_or_else(PoisonError::into_inner);
    context.map(|SharedContext(context)| context)
}

/// Initializes the [`AndroidContext`]. [`AndroidContext`] is initialized by [__ndk-glue__](https://crates.io/crates/ndk-glue)
/// before `main` is called.
///
/// # Safety
///
/// The pointers must be valid and this function must be called exactly once before `main` is
/// called.
///
/// # Panics
///
/// Panics if the context is already initialized.
pub unsafe fn initialize_android_context(java_vm: *mut c_void, context_jobject: *mut c_void) {
    // SAFETY: the caller upholds the same contract.
    let initialized = try_initialize_android_context(java_vm, context_jobject);
    assert!(
        initialized.is_ok(),
        "android context was already initialized"
    );
}

/// Initializes the [`AndroidContext`], or returns the one already set.
///
/// # Safety
///
/// The pointers must be valid.
pub unsafe fn try_initialize_android_context(
    java_vm: *mut c_void,
    context_jobject: *mut c_void,
) -> Result<(), AndroidContext> {
    let mut slot = ANDROID_CONTEXT
        .write()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(SharedContext(existing)) = *slot {
        return Err(existing);
    }
    *slot = Some(SharedContext(AndroidContext {
        java_vm,
        context_jobject,
    }));
    Ok(())
}

/// Removes the [`AndroidContext`]. It is released by [__ndk-glue__](https://crates.io/crates/ndk-glue)
/// when the activity is finished and destroyed.
///
/// # Safety
///
/// This function must only be called after [`initialize_android_context()`],
/// when the activity is subsequently destroyed according to Android.
///
/// # Panics
///
/// Panics if the context is not initialized.
pub unsafe fn release_android_context() {
    // SAFETY: the caller upholds the same contract.
    let released = try_release_android_context();
    assert!(released.is_some(), "android context was not initialized");
}

/// Removes and returns the [`AndroidContext`], if any.
///
/// # Safety
///
/// Must only be called when the activity is destroyed according to Android.
pub unsafe fn try_release_android_context() -> Option<AndroidContext> {
    let released = ANDROID_CONTEXT
        .write()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    released.map(|SharedContext(context)| context)
}
