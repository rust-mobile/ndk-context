use std::{any::Any, ffi::c_void, panic, ptr::NonNull};

fn dangling<T>() -> *mut c_void {
    NonNull::<T>::dangling().as_ptr().cast()
}

fn message(payload: Box<dyn Any + Send>) -> String {
    match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => (*payload.downcast::<&str>().unwrap()).to_owned(),
    }
}

#[test]
fn panicking_functions_keep_the_first_context() {
    let (first_vm, first_context) = (dangling::<u8>(), dangling::<u16>());
    let (second_vm, second_context) = (dangling::<u32>(), dangling::<u64>());

    let unset = panic::catch_unwind(ndk_context::android_context).unwrap_err();
    assert_eq!(message(unset), "android context was not initialized");

    // SAFETY: nothing in this test dereferences the dangling pointers.
    unsafe { ndk_context::initialize_android_context(first_vm, first_context) };
    let second = panic::catch_unwind(|| unsafe {
        ndk_context::initialize_android_context(second_vm, second_context)
    })
    .unwrap_err();
    assert_eq!(message(second), "android context was already initialized");
    let kept = ndk_context::android_context();
    assert_eq!((kept.vm(), kept.context()), (first_vm, first_context));

    // SAFETY: as above.
    unsafe { ndk_context::release_android_context() };
    let released_twice =
        panic::catch_unwind(|| unsafe { ndk_context::release_android_context() }).unwrap_err();
    assert_eq!(
        message(released_twice),
        "android context was not initialized"
    );
}
