//! Readers racing the writers must see either the context or nothing, checked for data races by
//! `MIRIFLAGS="-Zmiri-many-seeds=0..32" cargo +nightly miri test -p ndk-context`.

use std::{ffi::c_void, ptr::NonNull, thread};

fn fake_vm() -> *mut c_void {
    NonNull::<u8>::dangling().as_ptr().cast()
}

fn fake_context() -> *mut c_void {
    NonNull::<u16>::dangling().as_ptr().cast()
}

fn spawn_readers() -> Vec<thread::JoinHandle<()>> {
    (0..2)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..2 {
                    if let Some(context) = ndk_context::try_android_context() {
                        assert_eq!(context.vm(), fake_vm());
                        assert_eq!(context.context(), fake_context());
                    }
                }
            })
        })
        .collect()
}

fn join(readers: Vec<thread::JoinHandle<()>>) {
    for reader in readers {
        reader.join().unwrap();
    }
}

#[test]
fn readers_race_initialization_and_release() {
    let readers = spawn_readers();
    // SAFETY: nothing in this test dereferences the dangling pointers.
    unsafe { ndk_context::initialize_android_context(fake_vm(), fake_context()) };
    join(readers);

    let readers = spawn_readers();
    // SAFETY: the context was initialised above and no reader dereferences it.
    unsafe { ndk_context::release_android_context() };
    join(readers);
}
