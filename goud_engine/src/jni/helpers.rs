#![allow(dead_code)]

use jni::objects::{JByteArray, JClass, JLongArray, JObject, JString, JValue};
use jni::sys::{
    jboolean, jbyteArray, jint, jlong, jlongArray, jobject, jstring, JNI_FALSE, JNI_TRUE,
};
use jni::JNIEnv;

use crate::ffi::error::{goud_clear_last_error, goud_last_error_code, goud_last_error_message};

pub(crate) type JniCallResult<T> = Result<T, ()>;

pub(crate) fn clear_last_error() {
    goud_clear_last_error();
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
