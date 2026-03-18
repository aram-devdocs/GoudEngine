use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use jni::objects::{JObject, JString, JValue};
use jni::{InitArgsBuilder, JavaVM};

use super::{generated, helpers};

static JVM: OnceLock<JavaVM> = OnceLock::new();

fn compile_java_fixtures() -> PathBuf {
    static CLASSES_DIR: OnceLock<PathBuf> = OnceLock::new();
    CLASSES_DIR
        .get_or_init(|| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let source_dir = manifest_dir.join("tests/jni/java/com/goudengine/internal");
            let classes_dir = manifest_dir.join("../target/jni-rust-test-classes");

            std::fs::create_dir_all(&classes_dir).expect("failed to create JNI test class dir");

            let mut sources = std::fs::read_dir(&source_dir)
                .expect("failed to read JNI fixture directory")
                .map(|entry| entry.expect("invalid JNI fixture entry").path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "java"))
                .collect::<Vec<_>>();
            sources.sort();

            let status = Command::new("javac")
                .arg("-d")
                .arg(&classes_dir)
                .args(&sources)
                .status()
                .expect("failed to execute javac for JNI fixture classes");
            assert!(
                status.success(),
                "javac failed for JNI fixture classes: {status}"
            );

            classes_dir
        })
        .clone()
}

fn jvm() -> &'static JavaVM {
    JVM.get_or_init(|| {
        let classes_dir = compile_java_fixtures();
        let classpath = format!("-Djava.class.path={}", classes_dir.display());
        let args = InitArgsBuilder::new()
            .option(&classpath)
            .build()
            .expect("failed to build JVM init args");
        JavaVM::new(args).expect("failed to create JVM for JNI tests")
    })
}

fn take_exception_message(env: &mut jni::JNIEnv<'_>, expected_class: &str) -> String {
    assert!(
        env.exception_check()
            .expect("failed to check JVM exception"),
        "expected pending JVM exception"
    );
    let throwable = env
        .exception_occurred()
        .expect("failed to fetch pending JVM exception");
    env.exception_clear()
        .expect("failed to clear pending JVM exception");
    assert!(
        env.is_instance_of(&throwable, expected_class)
            .expect("failed to test JVM exception class"),
        "expected exception type {expected_class}"
    );

    let message_obj = env
        .call_method(&throwable, "getMessage", "()Ljava/lang/String;", &[])
        .expect("failed to call Throwable.getMessage")
        .l()
        .expect("Throwable.getMessage did not return an object");
    if message_obj.is_null() {
        return String::new();
    }

    env.get_string(&JString::from(message_obj))
        .expect("failed to read exception message")
        .into()
}

fn assert_color_eq(actual: crate::ffi::FfiColor, expected: crate::ffi::FfiColor) {
    assert!((actual.r - expected.r).abs() < 0.0001);
    assert!((actual.g - expected.g).abs() < 0.0001);
    assert!((actual.b - expected.b).abs() < 0.0001);
    assert!((actual.a - expected.a).abs() < 0.0001);
}

#[test]
fn require_string_bytes_round_trips_utf8() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    let value = env
        .new_string("jni-hello")
        .expect("failed to allocate Java string");
    let bytes = helpers::require_string_bytes(&mut env, value, "value")
        .expect("string bytes should round-trip");
    assert_eq!(bytes, b"jni-hello");
}

#[test]
fn require_c_string_round_trips_utf8() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    let value = env
        .new_string("jni-bridge")
        .expect("failed to allocate Java string");
    let c_string = helpers::require_c_string(&mut env, value, "value")
        .expect("CString conversion should succeed");
    assert_eq!(
        c_string.to_str().expect("CString should remain UTF-8"),
        "jni-bridge"
    );
}

#[test]
fn require_string_bytes_rejects_null_reference() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    let value = JString::from(JObject::null());

    assert!(helpers::require_string_bytes(&mut env, value, "value").is_err());

    let message = take_exception_message(&mut env, "java/lang/NullPointerException");
    assert!(message.contains("value is null"));
}

#[test]
fn ensure_no_pending_exception_short_circuits() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    env.throw_new("java/lang/IllegalStateException", "pending failure")
        .expect("failed to throw test exception");

    assert!(helpers::ensure_no_pending_exception(&mut env).is_err());

    let message = take_exception_message(&mut env, "java/lang/IllegalStateException");
    assert_eq!(message, "pending failure");
}

#[test]
fn prepare_call_short_circuits_before_clearing_last_error() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    crate::core::error::set_last_error(crate::core::error::GoudError::InvalidContext);
    env.throw_new("java/lang/IllegalStateException", "pending prepare failure")
        .expect("failed to throw test exception");

    assert!(helpers::prepare_call(&mut env).is_err());
    assert_ne!(helpers::last_error_code(), 0);

    let message = take_exception_message(&mut env, "java/lang/IllegalStateException");
    assert_eq!(message, "pending prepare failure");
    helpers::clear_last_error();
}

#[test]
fn throw_engine_error_uses_thread_local_error_text() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    helpers::clear_last_error();
    crate::core::error::set_last_error(crate::core::error::GoudError::InvalidState(
        "network bridge failed".to_string(),
    ));

    helpers::throw_engine_error(&mut env, "goud_network_send", Some(-3))
        .expect("throw_engine_error should raise a Java exception");

    let message = take_exception_message(&mut env, "java/lang/IllegalStateException");
    assert!(message.contains("goud_network_send"));
    assert!(message.contains("network bridge failed"));
}

#[test]
fn checked_output_length_rejects_oversized_writes() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");

    assert!(helpers::checked_output_length(&mut env, "jni_test", "outResults", 5, 4).is_err());

    let message = take_exception_message(&mut env, "java/lang/IllegalStateException");
    assert!(message.contains("jni_test"));
    assert!(message.contains("outResults"));
    assert!(message.contains("exceeds buffer capacity"));
}

#[test]
fn catch_jni_panic_turns_panics_into_java_exceptions() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");

    let result = helpers::catch_jni_panic(
        &mut env,
        "jni_test_panic",
        -7_i32,
        |_env| -> helpers::JniCallResult<i32> {
            panic!("panic boundary reached");
        },
    );

    assert_eq!(result, -7);

    let message = take_exception_message(&mut env, "java/lang/IllegalStateException");
    assert!(message.contains("jni_test_panic"));
    assert!(message.contains("panic boundary reached"));
}

#[test]
fn generated_color_carrier_round_trips_and_writes_back() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    let initial = crate::ffi::FfiColor {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 1.0,
    };
    let updated = crate::ffi::FfiColor {
        r: 0.9,
        g: 0.2,
        b: 0.1,
        a: 0.4,
    };

    let color =
        generated::new_Color(&mut env, initial).expect("failed to marshal Color carrier into Java");
    assert_color_eq(
        generated::read_Color(&mut env, &color, "color")
            .expect("failed to read Color carrier from Java"),
        initial,
    );

    generated::write_back_Color(&mut env, &color, updated)
        .expect("failed to write back Color carrier fields");
    assert_color_eq(
        generated::read_Color(&mut env, &color, "color")
            .expect("failed to read written Color carrier from Java"),
        updated,
    );
}

#[test]
fn generated_sprite_animator_rejects_invalid_enum_discriminant() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");
    let class = env
        .find_class("com/goudengine/internal/SpriteAnimator")
        .expect("failed to find SpriteAnimator fixture class");
    let sprite_animator = env
        .new_object(class, "()V", &[])
        .expect("failed to create SpriteAnimator fixture");
    env.set_field(&sprite_animator, "mode", "I", JValue::Int(99))
        .expect("failed to inject invalid enum discriminant");

    assert!(generated::read_SpriteAnimator(&mut env, &sprite_animator, "spriteAnimator").is_err());

    let message = take_exception_message(&mut env, "java/lang/IllegalArgumentException");
    assert!(message.contains("SpriteAnimator.mode"));
    assert!(message.contains("invalid"));
}

#[test]
fn read_fixed_buffer_string_rejects_oversized_lengths() {
    let mut env = jvm()
        .attach_current_thread()
        .expect("failed to attach JNI test thread");

    let result = generated::read_fixed_buffer_string(
        &mut env,
        "jni_test_fixed_buffer",
        |_buf, len| len + 1,
        4,
    );
    assert!(result.is_err());

    let message = take_exception_message(&mut env, "java/lang/IllegalStateException");
    assert!(message.contains("jni_test_fixed_buffer"));
    assert!(message.contains("buffer"));
}
