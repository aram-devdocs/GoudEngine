use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn classes_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "goud-engine-jni-integration-classes-{}",
        std::process::id()
    ))
}

fn java_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/jni/java/com/goudengine/internal")
}

fn compile_java_fixtures() -> PathBuf {
    let classes_dir = classes_dir();
    if classes_dir.exists() {
        fs::remove_dir_all(&classes_dir).expect("remove stale JNI smoke classes");
    }
    fs::create_dir_all(&classes_dir).expect("create JNI smoke class directory");

    let mut java_files: Vec<PathBuf> = fs::read_dir(java_fixture_dir())
        .expect("read JNI smoke fixture directory")
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("java"))
        .collect();
    java_files.sort();

    let output = Command::new("javac")
        .arg("-d")
        .arg(&classes_dir)
        .args(&java_files)
        .output()
        .expect("run javac for JNI smoke fixtures");
    assert!(
        output.status.success(),
        "javac failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    classes_dir
}

fn shared_library_path() -> PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    target_dir.join("debug").join(shared_library_name())
}

#[cfg(target_os = "macos")]
fn shared_library_name() -> &'static str {
    "libgoud_engine.dylib"
}

#[cfg(target_os = "linux")]
fn shared_library_name() -> &'static str {
    "libgoud_engine.so"
}

#[cfg(target_os = "windows")]
fn shared_library_name() -> &'static str {
    "goud_engine.dll"
}

#[test]
fn test_jni_smoke_main_loads_shared_library() {
    let classes_dir = compile_java_fixtures();
    let library_path = shared_library_path();
    assert!(
        library_path.is_file(),
        "expected JNI shared library at {}",
        library_path.display()
    );

    let output = Command::new("java")
        .arg("-cp")
        .arg(&classes_dir)
        .arg("com.goudengine.internal.JniSmokeMain")
        .arg(&library_path)
        .output()
        .expect("run Java JNI smoke main");
    assert!(
        output.status.success(),
        "JNI smoke run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
