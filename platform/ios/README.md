# iOS Build Pipeline

Run `./platform/ios/build-ios.sh` from the repo root to build the Rust static library for iOS.

The script:

- installs `cargo-lipo` if it is missing
- installs the required Rust iOS targets
- builds a device archive and a universal simulator archive
- stages `goud_engine.h` next to the packaged libraries

Output lands in `platform/ios/build/`:

- `device/libgoud_engine.a`
- `simulator/libgoud_engine.a`
- `include/goud_engine.h`

The template app in `platform/ios/template/` uses the existing Swift SDK package and expects the simulator archive at `platform/ios/build/simulator/libgoud_engine.a`.

Build the template for the simulator with:

```bash
./platform/ios/build-ios.sh
xcodebuild \
  -project platform/ios/template/MobileTemplate.xcodeproj \
  -scheme MobileTemplate \
  -configuration Debug \
  -sdk iphonesimulator \
  -destination 'generic/platform=iOS Simulator' \
  build
```
