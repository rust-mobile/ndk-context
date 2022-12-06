use android_activity::AndroidApp;
use jni::objects::{JObject, JValue};
use log::info;

/// A minimal example of how to use `ndk_context` to get a `JavaVM` + `Context and make a JNI call
fn ndk_context_jni_test() -> Result<(), Box<dyn std::error::Error>> {
    // Get a VM for executing JNI calls
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let _context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let env = vm.attach_current_thread()?;

    // Since we aren't making JNI calls within the implementation of a native call from the JavaVM
    // we wrap the reference in an `AutoLocal` to make sure it will be deleted.
    let _int_ref = env.auto_local(
        env.new_object("java/lang/Integer", "(I)V", &[JValue::Int(42)])
            .unwrap(),
    );

    Ok(())
}

#[no_mangle]
fn android_main(_app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));

    ndk_context_jni_test().unwrap();

    info!("hello world");
    println!("hello world");
}
