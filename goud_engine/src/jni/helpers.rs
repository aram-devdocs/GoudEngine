#![allow(dead_code)]

use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use jni::objects::{JByteArray, JClass, JFloatArray, JLongArray, JObject, JString, JValue};
use jni::sys::{
    jboolean, jbyteArray, jdouble, jint, jlong, jlongArray, jobject, jstring, JNI_FALSE, JNI_TRUE,
};
use jni::JNIEnv;

use crate::ffi::error::{goud_clear_last_error, goud_last_error_code, goud_last_error_message};

pub(crate) type JniCallResult<T> = Result<T, ()>;

pub(crate) fn clear_last_error() {
    goud_clear_last_error();
}

pub(crate) fn prepare_call(env: &mut JNIEnv<'_>) -> JniCallResult<()> {
    ensure_no_pending_exception(env)?;
    clear_last_error();
    Ok(())
}

pub(crate) fn to_jboolean(value: bool) -> jboolean {
    if value {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

pub(crate) fn from_jboolean(value: jboolean) -> bool {
    value != JNI_FALSE
}

pub(crate) fn last_error_code() -> jint {
    goud_last_error_code()
}

pub(crate) fn last_error_message() -> String {
    let required =
        // SAFETY: querying with a null buffer is the documented size-discovery protocol.
        unsafe { goud_last_error_message(std::ptr::null_mut(), 0) };
    if required >= 0 {
        return String::new();
    }
    let size = (-required) as usize;
    let mut buf = vec![0u8; size];
    let written =
        // SAFETY: `buf` is a valid writable buffer of `size` bytes.
        unsafe { goud_last_error_message(buf.as_mut_ptr(), buf.len()) };
    if written <= 0 {
        return String::new();
    }
    String::from_utf8_lossy(&buf[..written as usize]).into_owned()
}

pub(crate) fn throw_null_pointer(
    env: &mut JNIEnv<'_>,
    message: impl AsRef<str>,
) -> JniCallResult<()> {
    env.throw_new("java/lang/NullPointerException", message.as_ref())
        .map_err(|_| ())
}

pub(crate) fn throw_illegal_argument(
    env: &mut JNIEnv<'_>,
    message: impl AsRef<str>,
) -> JniCallResult<()> {
    env.throw_new("java/lang/IllegalArgumentException", message.as_ref())
        .map_err(|_| ())
}

pub(crate) fn throw_illegal_state(
    env: &mut JNIEnv<'_>,
    message: impl AsRef<str>,
) -> JniCallResult<()> {
    env.throw_new("java/lang/IllegalStateException", message.as_ref())
        .map_err(|_| ())
}

pub(crate) fn throw_engine_error(
    env: &mut JNIEnv<'_>,
    function_name: &str,
    status: Option<i64>,
) -> JniCallResult<()> {
    let message = last_error_message();
    let rendered = if message.is_empty() {
        match status {
            Some(code) => format!("{function_name} failed with status {code}."),
            None => format!("{function_name} failed."),
        }
    } else {
        match status {
            Some(code) => format!("{function_name} failed with status {code}: {message}"),
            None => format!("{function_name} failed: {message}"),
        }
    };
    throw_illegal_state(env, rendered)
}

pub(crate) fn throw_java_error(
    env: &mut JNIEnv<'_>,
    function_name: &str,
    message: impl AsRef<str>,
) -> JniCallResult<()> {
    throw_illegal_state(env, format!("{function_name}: {}", message.as_ref()))
}

pub(crate) fn checked_output_length(
    env: &mut JNIEnv<'_>,
    function_name: &str,
    target_name: &str,
    written: usize,
    capacity: usize,
) -> JniCallResult<usize> {
    if written > capacity {
        throw_java_error(
            env,
            function_name,
            format!("{target_name} length {written} exceeds buffer capacity {capacity}"),
        )?;
        return Err(());
    }
    Ok(written)
}

pub(crate) fn throw_panic(
    env: &mut JNIEnv<'_>,
    function_name: &str,
    payload: Box<dyn Any + Send>,
) -> JniCallResult<()> {
    if env.exception_check().unwrap_or(false) {
        return Err(());
    }
    let message = match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => match payload.downcast::<&'static str>() {
            Ok(message) => (*message).to_string(),
            Err(_) => "unknown panic payload".to_string(),
        },
    };
    throw_illegal_state(env, format!("{function_name} panicked: {message}"))
}

pub(crate) fn catch_jni_panic<'local, T, F>(
    env: &mut JNIEnv<'local>,
    function_name: &str,
    default: T,
    body: F,
) -> T
where
    F: FnOnce(&mut JNIEnv<'local>) -> JniCallResult<T>,
{
    match catch_unwind(AssertUnwindSafe(|| body(env))) {
        Ok(Ok(value)) => value,
        Ok(Err(())) => default,
        Err(payload) => {
            let _ = throw_panic(env, function_name, payload);
            default
        }
    }
}

pub(crate) fn ensure_no_pending_exception(env: &mut JNIEnv<'_>) -> JniCallResult<()> {
    match env.exception_check() {
        Ok(true) => Err(()),
        Ok(false) => Ok(()),
        Err(_) => Err(()),
    }
}

pub(crate) fn require_object<'local>(
    env: &mut JNIEnv<'local>,
    obj: JObject<'local>,
    param_name: &str,
) -> JniCallResult<JObject<'local>> {
    if obj.is_null() {
        throw_null_pointer(env, format!("{param_name} is null"))?;
        return Err(());
    }
    Ok(obj)
}

pub(crate) fn require_string_bytes(
    env: &mut JNIEnv<'_>,
    value: JString<'_>,
    param_name: &str,
) -> JniCallResult<Vec<u8>> {
    if value.is_null() {
        throw_null_pointer(env, format!("{param_name} is null"))?;
        return Err(());
    }
    let java_str = env.get_string(&value).map_err(|_| ())?;
    Ok(java_str.to_str().map_err(|_| ())?.as_bytes().to_vec())
}

pub(crate) fn require_c_string(
    env: &mut JNIEnv<'_>,
    value: JString<'_>,
    param_name: &str,
) -> JniCallResult<std::ffi::CString> {
    let bytes = require_string_bytes(env, value, param_name)?;
    std::ffi::CString::new(bytes).map_err(|_| ()).or_else(|_| {
        throw_illegal_argument(env, format!("{param_name} contains an interior NUL"))?;
        Err(())
    })
}

pub(crate) fn require_bytes(
    env: &mut JNIEnv<'_>,
    array: JByteArray<'_>,
    param_name: &str,
) -> JniCallResult<Vec<u8>> {
    if array.is_null() {
        throw_null_pointer(env, format!("{param_name} is null"))?;
        return Err(());
    }
    env.convert_byte_array(array).map_err(|_| ())
}

pub(crate) fn byte_array_length(
    env: &mut JNIEnv<'_>,
    array: &JByteArray<'_>,
) -> JniCallResult<usize> {
    env.get_array_length(array)
        .map(|length| length as usize)
        .map_err(|_| ())
}

pub(crate) fn write_byte_array(
    env: &mut JNIEnv<'_>,
    array: &JByteArray<'_>,
    bytes: &[u8],
) -> JniCallResult<()> {
    let values: Vec<i8> = bytes.iter().map(|value| *value as i8).collect();
    env.set_byte_array_region(array, 0, &values).map_err(|_| ())
}

pub(crate) fn require_long_array(
    env: &mut JNIEnv<'_>,
    array: jlongArray,
    param_name: &str,
) -> JniCallResult<Vec<jlong>> {
    if array.is_null() {
        throw_null_pointer(env, format!("{param_name} is null"))?;
        return Err(());
    }
    let array =
        // SAFETY: `array` is a live JNI reference provided by the JVM for this call and was checked for null above.
        unsafe { JLongArray::from_raw(array) };
    let length = env.get_array_length(&array).map_err(|_| ())? as usize;
    let mut values = vec![0_i64; length];
    env.get_long_array_region(&array, 0, &mut values)
        .map_err(|_| ())?;
    Ok(values)
}

pub(crate) fn new_java_string(env: &mut JNIEnv<'_>, value: &str) -> JniCallResult<jstring> {
    env.new_string(value).map(|s| s.into_raw()).map_err(|_| ())
}

pub(crate) fn new_byte_array(env: &mut JNIEnv<'_>, bytes: &[u8]) -> JniCallResult<jbyteArray> {
    env.byte_array_from_slice(bytes)
        .map(|array| array.into_raw())
        .map_err(|_| ())
}

pub(crate) fn new_long_array(env: &mut JNIEnv<'_>, values: &[i64]) -> JniCallResult<jlongArray> {
    let array = env.new_long_array(values.len() as i32).map_err(|_| ())?;
    env.set_long_array_region(&array, 0, values)
        .map_err(|_| ())?;
    Ok(array.into_raw())
}

pub(crate) fn null_object() -> jobject {
    std::ptr::null_mut()
}

pub(crate) fn null_string() -> jstring {
    std::ptr::null_mut()
}

pub(crate) fn null_byte_array() -> jbyteArray {
    std::ptr::null_mut()
}

pub(crate) fn find_class<'local>(
    env: &mut JNIEnv<'local>,
    class_name: &str,
) -> JniCallResult<JClass<'local>> {
    env.find_class(class_name).map_err(|_| ())
}

pub(crate) fn new_object<'local>(
    env: &mut JNIEnv<'local>,
    class_name: &str,
) -> JniCallResult<JObject<'local>> {
    let class = find_class(env, class_name)?;
    env.new_object(class, "()V", &[]).map_err(|_| ())
}

pub(crate) fn new_hash_map<'local>(env: &mut JNIEnv<'local>) -> JniCallResult<JObject<'local>> {
    new_object(env, "java/util/HashMap")
}

pub(crate) fn put_hash_map_value(
    env: &mut JNIEnv<'_>,
    map: &JObject<'_>,
    key: &str,
    value: &JObject<'_>,
) -> JniCallResult<()> {
    let key = env.new_string(key).map_err(|_| ())?;
    let key_obj = JObject::from(key);
    env.call_method(
        map,
        "put",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        &[JValue::Object(&key_obj), JValue::Object(value)],
    )
    .map_err(|_| ())?;
    Ok(())
}

pub(crate) fn new_boxed_long<'local>(
    env: &mut JNIEnv<'local>,
    value: jlong,
) -> JniCallResult<JObject<'local>> {
    let class = find_class(env, "java/lang/Long")?;
    env.new_object(class, "(J)V", &[JValue::Long(value)])
        .map_err(|_| ())
}

pub(crate) fn new_boxed_int<'local>(
    env: &mut JNIEnv<'local>,
    value: jint,
) -> JniCallResult<JObject<'local>> {
    let class = find_class(env, "java/lang/Integer")?;
    env.new_object(class, "(I)V", &[JValue::Int(value)])
        .map_err(|_| ())
}

pub(crate) fn new_boxed_float<'local>(
    env: &mut JNIEnv<'local>,
    value: f32,
) -> JniCallResult<JObject<'local>> {
    let class = find_class(env, "java/lang/Float")?;
    env.new_object(class, "(F)V", &[JValue::Float(value)])
        .map_err(|_| ())
}

pub(crate) fn new_boxed_double<'local>(
    env: &mut JNIEnv<'local>,
    value: jdouble,
) -> JniCallResult<JObject<'local>> {
    let class = find_class(env, "java/lang/Double")?;
    env.new_object(class, "(D)V", &[JValue::Double(value)])
        .map_err(|_| ())
}

pub(crate) fn get_boolean_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
) -> JniCallResult<bool> {
    env.get_field(object, field_name, "Z")
        .and_then(|value| value.z())
        .map_err(|_| ())
}

pub(crate) fn set_boolean_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    value: bool,
) -> JniCallResult<()> {
    env.set_field(object, field_name, "Z", JValue::Bool(to_jboolean(value)))
        .map_err(|_| ())
}

pub(crate) fn get_int_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
) -> JniCallResult<jint> {
    env.get_field(object, field_name, "I")
        .and_then(|value| value.i())
        .map_err(|_| ())
}

pub(crate) fn set_int_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    value: jint,
) -> JniCallResult<()> {
    env.set_field(object, field_name, "I", JValue::Int(value))
        .map_err(|_| ())
}

pub(crate) fn get_long_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
) -> JniCallResult<jlong> {
    env.get_field(object, field_name, "J")
        .and_then(|value| value.j())
        .map_err(|_| ())
}

pub(crate) fn set_long_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    value: jlong,
) -> JniCallResult<()> {
    env.set_field(object, field_name, "J", JValue::Long(value))
        .map_err(|_| ())
}

pub(crate) fn get_float_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
) -> JniCallResult<f32> {
    env.get_field(object, field_name, "F")
        .and_then(|value| value.f())
        .map_err(|_| ())
}

pub(crate) fn set_float_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    value: f32,
) -> JniCallResult<()> {
    env.set_field(object, field_name, "F", JValue::Float(value))
        .map_err(|_| ())
}

pub(crate) fn get_double_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
) -> JniCallResult<jdouble> {
    env.get_field(object, field_name, "D")
        .and_then(|value| value.d())
        .map_err(|_| ())
}

pub(crate) fn set_double_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    value: jdouble,
) -> JniCallResult<()> {
    env.set_field(object, field_name, "D", JValue::Double(value))
        .map_err(|_| ())
}

pub(crate) fn get_string_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
) -> JniCallResult<String> {
    let raw = env
        .get_field(object, field_name, "Ljava/lang/String;")
        .and_then(|value| value.l())
        .map_err(|_| ())?;
    require_string_bytes(env, JString::from(raw), field_name)
        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
}

pub(crate) fn set_string_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    value: &str,
) -> JniCallResult<()> {
    let string = env.new_string(value).map_err(|_| ())?;
    env.set_field(
        object,
        field_name,
        "Ljava/lang/String;",
        JValue::Object(&JObject::from(string)),
    )
    .map_err(|_| ())
}

pub(crate) fn get_object_field<'local>(
    env: &mut JNIEnv<'local>,
    object: &JObject<'local>,
    field_name: &str,
    signature: &str,
) -> JniCallResult<JObject<'local>> {
    env.get_field(object, field_name, signature)
        .and_then(|value| value.l())
        .map_err(|_| ())
}

pub(crate) fn set_object_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    signature: &str,
    value: &JObject<'_>,
) -> JniCallResult<()> {
    env.set_field(object, field_name, signature, JValue::Object(value))
        .map_err(|_| ())
}

pub(crate) fn set_byte_array_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    bytes: &[u8],
) -> JniCallResult<()> {
    let array = env.byte_array_from_slice(bytes).map_err(|_| ())?;
    let array_obj = JObject::from(array);
    set_object_field(env, object, field_name, "[B", &array_obj)
}

pub(crate) fn set_float_array_field(
    env: &mut JNIEnv<'_>,
    object: &JObject<'_>,
    field_name: &str,
    values: &[f32],
) -> JniCallResult<()> {
    let array = env.new_float_array(values.len() as i32).map_err(|_| ())?;
    env.set_float_array_region(&array, 0, values)
        .map_err(|_| ())?;
    let array_obj = JObject::from(array);
    set_object_field(env, object, field_name, "[F", &array_obj)
}

pub(crate) fn get_float_array_field<'local, const N: usize>(
    env: &mut JNIEnv<'local>,
    object: &JObject<'local>,
    field_name: &str,
) -> JniCallResult<[f32; N]> {
    let field_obj = get_object_field(env, object, field_name, "[F")?;
    let array = JFloatArray::from(field_obj);
    let mut values = [0.0f32; N];
    env.get_float_array_region(&array, 0, &mut values)
        .map_err(|_| ())?;
    Ok(values)
}
