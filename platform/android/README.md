# Android Build Pipeline

Run `./platform/android/build-android.sh` from the repo root to build the Rust shared library for Android.

The script:

- installs `cargo-ndk` if it is missing
- installs the required Rust Android targets
- resolves the Android NDK from `ANDROID_NDK_HOME`, `ANDROID_NDK_ROOT`, or the default SDK path
- builds `libgoud_engine.so` for `arm64-v8a` and `x86_64`
- stages `goud_engine.h` next to the packaged libraries

The ABI-specific `.so` files are copied into `platform/android/template/app/src/main/jniLibs/`.

Build the template with:

```bash
./platform/android/build-android.sh
cd platform/android/template
./gradlew assembleDebug --no-daemon
```
