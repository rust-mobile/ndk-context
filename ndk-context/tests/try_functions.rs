use std::{ffi::c_void, ptr::NonNull};

fn dangling<T>() -> *mut c_void {
    NonNull::<T>::dangling().as_ptr().cast()
}

#[test]
fn try_functions_report_instead_of_panicking() {
    let (first_vm, first_context) = (dangling::<u8>(), dangling::<u16>());
    let (second_vm, second_context) = (dangling::<u32>(), dangling::<u64>());

    assert!(ndk_context::try_android_context().is_none());
    // SAFETY: nothing in this test dereferences the dangling pointers.
    assert!(unsafe { ndk_context::try_release_android_context() }.is_none());

    // SAFETY: as above.
    assert!(
        unsafe { ndk_context::try_initialize_android_context(first_vm, first_context) }.is_ok()
    );
    // SAFETY: as above.
    let kept = unsafe { ndk_context::try_initialize_android_context(second_vm, second_context) }
        .unwrap_err();
    assert_eq!((kept.vm(), kept.context()), (first_vm, first_context));

    let current = ndk_context::try_android_context().unwrap();
    assert_eq!((current.vm(), current.context()), (first_vm, first_context));

    // SAFETY: as above.
    let released = unsafe { ndk_context::try_release_android_context() }.unwrap();
    assert_eq!(
        (released.vm(), released.context()),
        (first_vm, first_context)
    );
    assert!(ndk_context::try_android_context().is_none());
}
