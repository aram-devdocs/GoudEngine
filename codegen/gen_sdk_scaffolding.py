#!/usr/bin/env python3
"""Generates SDK package/build scaffolding from canonical repo metadata."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    GENERATED_BY,
    HEADER_COMMENT,
    SDKS_DIR,
    load_repo_version,
    write_generated,
    write_generated_json,
)

VERSION = load_repo_version()
REPO_URL = "https://github.com/aram-devdocs/GoudEngine"
AUTHOR = "Aram Hammoudeh"


def gen_csharp_scaffolding() -> None:
    csproj = "\n".join([
        f"<!-- {HEADER_COMMENT} -->",
        "<Project Sdk=\"Microsoft.NET.Sdk\">",
        "",
        "  <PropertyGroup>",
        "    <TargetFramework>net8.0</TargetFramework>",
        "    <Nullable>enable</Nullable>",
        "    <ImplicitUsings>disable</ImplicitUsings>",
        "    <GeneratePackageOnBuild>true</GeneratePackageOnBuild>",
        "    <PackageId>GoudEngine</PackageId>",
        f"    <Version>{VERSION}</Version> <!-- x-release-please-version -->",
        f"    <Authors>{AUTHOR}</Authors>",
        "    <Description>Rust-powered game engine for .NET. Build 2D and 3D games with C#.</Description>",
        "    <PackageLicenseExpression>MIT</PackageLicenseExpression>",
        "    <PackageOutputPath>../nuget_package_output</PackageOutputPath>",
        "    <PackageReadmeFile>README.md</PackageReadmeFile>",
        f"    <RepositoryUrl>{REPO_URL}</RepositoryUrl>",
        "    <RepositoryType>git</RepositoryType>",
        "    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>",
        "    <GenerateDocumentationFile>true</GenerateDocumentationFile>",
        "    <NoWarn>$(NoWarn);CS1591</NoWarn>",
        "  </PropertyGroup>",
        "",
        "  <ItemGroup>",
        "    <None Include=\"runtimes/**\" Pack=\"true\" PackagePath=\"runtimes/\" />",
        "    <None Include=\"include/**\" Pack=\"true\" PackagePath=\"include/\" />",
        "    <None Include=\"README.md\" Pack=\"true\" PackagePath=\"/\" />",
        "  </ItemGroup>",
        "",
        "  <ItemGroup>",
        "    <Compile Remove=\"sdks/**/*.cs\" />",
        "  </ItemGroup>",
        "",
        "  <Import Project=\"build/GoudEngine.targets\" Condition=\"Exists('build/GoudEngine.targets')\" />",
        "",
        "</Project>",
        "",
    ])
    write_generated(SDKS_DIR / "csharp" / "GoudEngine.csproj", csproj)

    targets = "\n".join([
        f"<!-- {HEADER_COMMENT} -->",
        "<Project xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">",
        "",
        "  <Target Name=\"CopyGoudEngineNativeLib\" AfterTargets=\"Build\">",
        "",
        "    <PropertyGroup>",
        "      <GoudNativeDir>$(MSBuildThisFileDirectory)../runtimes</GoudNativeDir>",
        "      <GoudRepoRoot>$(MSBuildThisFileDirectory)../../..</GoudRepoRoot>",
        "      <GoudNativeLibrary></GoudNativeLibrary>",
        "    </PropertyGroup>",
        "",
        "    <PropertyGroup Condition=\"$([MSBuild]::IsOSPlatform('OSX'))\">",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'Arm64' And '$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)' == 'Arm64' And Exists('$(GoudRepoRoot)/target/debug/libgoud_engine.dylib')\">$(GoudRepoRoot)/target/debug/libgoud_engine.dylib</GoudNativeLibrary>",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'Arm64' And '$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)' == 'Arm64' And Exists('$(GoudRepoRoot)/target/release/libgoud_engine.dylib')\">$(GoudRepoRoot)/target/release/libgoud_engine.dylib</GoudNativeLibrary>",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'Arm64' And Exists('$(GoudNativeDir)/osx-arm64/native/libgoud_engine.dylib')\">$(GoudNativeDir)/osx-arm64/native/libgoud_engine.dylib</GoudNativeLibrary>",
        "",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'X64' And '$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)' == 'X64' And Exists('$(GoudRepoRoot)/target/debug/libgoud_engine.dylib')\">$(GoudRepoRoot)/target/debug/libgoud_engine.dylib</GoudNativeLibrary>",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'X64' And '$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)' == 'X64' And Exists('$(GoudRepoRoot)/target/release/libgoud_engine.dylib')\">$(GoudRepoRoot)/target/release/libgoud_engine.dylib</GoudNativeLibrary>",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'X64' And '$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)' == 'Arm64' And Exists('$(GoudRepoRoot)/target/x86_64-apple-darwin/debug/libgoud_engine.dylib')\">$(GoudRepoRoot)/target/x86_64-apple-darwin/debug/libgoud_engine.dylib</GoudNativeLibrary>",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'X64' And '$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)' == 'Arm64' And Exists('$(GoudRepoRoot)/target/x86_64-apple-darwin/release/libgoud_engine.dylib')\">$(GoudRepoRoot)/target/x86_64-apple-darwin/release/libgoud_engine.dylib</GoudNativeLibrary>",
        "      <GoudNativeLibrary Condition=\"'$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)' == 'X64' And Exists('$(GoudNativeDir)/osx-x64/native/libgoud_engine.dylib')\">$(GoudNativeDir)/osx-x64/native/libgoud_engine.dylib</GoudNativeLibrary>",
        "    </PropertyGroup>",
        "",
        "    <Message",
        "      Importance=\"high\"",
        "      Condition=\"'$(GoudNativeLibrary)' == ''\"",
        "      Text=\"No matching Darwin native library for architecture '$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)'. Expected: runtimes/osx-arm64/native/libgoud_engine.dylib for Arm64 or runtimes/osx-x64/native/libgoud_engine.dylib (optionally target/x86_64-apple-darwin/* for x64 Rosetta processes) for X64.\"",
        "    />",
        "",
        "    <PropertyGroup Condition=\"$([MSBuild]::IsOSPlatform('Linux'))\">",
        "      <GoudNativeLibrary Condition=\"Exists('$(GoudNativeDir)/linux-x64/native/libgoud_engine.so')\">$(GoudNativeDir)/linux-x64/native/libgoud_engine.so</GoudNativeLibrary>",
        "    </PropertyGroup>",
        "",
        "    <PropertyGroup Condition=\"$([MSBuild]::IsOSPlatform('Windows'))\">",
        "      <GoudNativeLibrary Condition=\"Exists('$(GoudNativeDir)/win-x64/native/goud_engine.dll')\">$(GoudNativeDir)/win-x64/native/goud_engine.dll</GoudNativeLibrary>",
        "    </PropertyGroup>",
        "",
        "    <Copy",
        "      SourceFiles=\"$(GoudNativeLibrary)\"",
        "      DestinationFolder=\"$(OutputPath)\"",
        "      Condition=\"'$(GoudNativeLibrary)' != ''\"",
        "      SkipUnchangedFiles=\"true\" />",
        "",
        "    <Message",
        "      Importance=\"low\"",
        "      Condition=\"'$(GoudNativeLibrary)' == ''\"",
        "      Text=\"GoudEngine native library not found under $(GoudNativeDir) for this OS.\" />",
        "",
        "  </Target>",
        "",
        "</Project>",
        "",
    ])
    write_generated(SDKS_DIR / "csharp" / "build" / "GoudEngine.targets", targets)


def gen_python_scaffolding() -> None:
    pyproject = "\n".join([
        f"# {HEADER_COMMENT}",
        "[build-system]",
        'requires = ["setuptools>=68.0", "wheel"]',
        'build-backend = "setuptools.build_meta"',
        "",
        "[project]",
        'name = "goudengine"',
        f'version = "{VERSION}" # x-release-please-version',
        'description = "Python SDK for GoudEngine. Build 2D and 3D games powered by a Rust core."',
        'readme = "README.md"',
        'license = { text = "MIT" }',
        'requires-python = ">=3.9"',
        f'authors = [{{ name = "{AUTHOR}" }}]',
        'keywords = ["game-engine", "ecs", "2d", "3d", "rust", "ffi"]',
        "classifiers = [",
        '    "Development Status :: 3 - Alpha",',
        '    "Intended Audience :: Developers",',
        '    "License :: OSI Approved :: MIT License",',
        '    "Programming Language :: Python :: 3",',
        '    "Programming Language :: Python :: 3.9",',
        '    "Programming Language :: Python :: 3.10",',
        '    "Programming Language :: Python :: 3.11",',
        '    "Programming Language :: Python :: 3.12",',
        '    "Programming Language :: Rust",',
        '    "Topic :: Games/Entertainment",',
        '    "Topic :: Software Development :: Libraries",',
        "]",
        "",
        "[project.urls]",
        f'Homepage = "{REPO_URL}"',
        f'Repository = "{REPO_URL}"',
        f'Issues = "{REPO_URL}/issues"',
        f'Documentation = "{REPO_URL}/tree/main/sdks/python"',
        "",
        "[tool.setuptools.packages.find]",
        'include = ["goud_engine*"]',
        "",
        "[tool.setuptools.package-data]",
        'goud_engine = ["*.so", "*.dylib", "*.dll", "include/*.h"]',
        "",
    ])
    write_generated(SDKS_DIR / "python" / "pyproject.toml", pyproject)

    manifest = "\n".join([
        f"# {HEADER_COMMENT}",
        "include goud_engine/*.so",
        "include goud_engine/*.dylib",
        "include goud_engine/*.dll",
        "include goud_engine/include/*.h",
        "include README.md",
        "",
    ])
    write_generated(SDKS_DIR / "python" / "MANIFEST.in", manifest)


def gen_typescript_scaffolding() -> None:
    typescript_dir = SDKS_DIR / "typescript"

    # wasm-pack already optimizes the web bundle. A second manual wasm-opt pass
    # regressed the browser networking runtime smoke in CI, so build:web must
    # stay on the direct wasm-pack output.
    package_json = {
        "name": "goudengine",
        "version": VERSION,
        "description": "GoudEngine - build 2D and 3D games with TypeScript for Node.js and web",
        "main": "dist/index.js",
        "types": "dist/index.d.ts",
        "sideEffects": False,
        "exports": {
            ".": {
                "node": {
                    "types": "./dist/index.d.ts",
                    "import": "./dist/index.js",
                    "require": "./dist/index.js",
                },
                "browser": {
                    "types": "./dist/web/index.d.ts",
                    "import": "./dist/web/index.js",
                },
                "default": {
                    "types": "./dist/index.d.ts",
                    "default": "./dist/index.js",
                },
            },
            "./web": {
                "types": "./dist/web/index.d.ts",
                "import": "./dist/web/index.js",
            },
            "./node": {
                "types": "./dist/node/index.d.ts",
                "import": "./dist/node/index.js",
            },
        },
        "scripts": {
            "build:native": "napi build --manifest-path native/Cargo.toml --platform --release --output-dir .",
            "build:native:debug": "napi build --manifest-path native/Cargo.toml --platform --output-dir .",
            "build:ts": "tsc",
            "build:ts:web": "tsc -p tsconfig.web.json",
            "build:wasm": "wasm-pack build ../../goud_engine --target web --out-dir ../sdks/typescript/wasm --features web --no-default-features",
            "build:web": "npm run build:wasm && npm run build:ts:web",
            "build": "npm run build:native && npm run build:ts",
            "build:all": "npm run build && npm run build:web",
            "clean": "rm -rf dist wasm *.node",
            "pretest": "npm run build:native:debug && npm run build:ts",
            "pretest:web-runtime": "npm run build:web",
            "typecheck": "tsc --noEmit && tsc -p tsconfig.web.json --noEmit",
            "test:native": "node --test test/errors.test.mjs test/generated-node-wrapper-coverage.test.mjs test/network-api.test.mjs test/network-loopback.test.mjs test/smoke.test.mjs",
            "test": "npm run test:native",
            "test:all": "npm run test:native && npm run test:web-runtime",
            "test:web-runtime": "node --test test/web-network-runtime-smoke.test.mjs",
            "precoverage:native": "npm run build:native:debug && npm run build:ts",
            "coverage:native": "c8 --reporter=text-summary --reporter=cobertura --report-dir coverage/native --check-coverage --lines 80 npm run test:native",
            "precoverage:web-runtime": "npm run build:web",
            "coverage:web-runtime": "node scripts/web-runtime-coverage.mjs",
            "prepublishOnly": "npm run clean && npm run build:all",
            "size": "node -e \"const fs=require('fs');try{const s=fs.statSync('wasm/goud_engine_bg.wasm');console.log((s.size/1024).toFixed(1)+'KB')}catch{console.log('No wasm build found')}\"",
        },
        "napi": {
            "binaryName": "goud-engine-node",
            "targets": [
                "x86_64-apple-darwin",
                "aarch64-apple-darwin",
                "x86_64-unknown-linux-gnu",
                "x86_64-pc-windows-msvc",
                "aarch64-unknown-linux-gnu",
                "aarch64-unknown-linux-musl",
            ],
        },
        "keywords": [
            "game-engine",
            "goudengine",
            "ecs",
            "napi-rs",
            "wasm",
            "webgpu",
            "webassembly",
            "2d",
            "3d",
        ],
        "license": "MIT",
        "repository": {
            "type": "git",
            "url": f"{REPO_URL}.git",
            "directory": "sdks/typescript",
        },
        "homepage": f"{REPO_URL}#readme",
        "bugs": {
            "url": f"{REPO_URL}/issues",
        },
        "devDependencies": {
            "@napi-rs/cli": "^3.5.1",
            "@types/node": "^20.0.0",
            "c8": "^10.1.3",
            "istanbul-lib-coverage": "^3.2.2",
            "istanbul-lib-report": "^3.0.1",
            "istanbul-reports": "^3.1.7",
            "playwright": "^1.52.0",
            "typescript": "^5.0.0",
            "v8-to-istanbul": "^9.3.0",
        },
        "engines": {
            "node": ">=16.0.0",
        },
        "files": [
            "dist",
            "wasm/*.js",
            "wasm/*.wasm",
            "wasm/*.d.ts",
            "index.js",
            "index.d.ts",
            "*.node",
        ],
        "publishConfig": {
            "access": "public",
        },
        "x-generatedBy": GENERATED_BY,
    }
    write_generated_json(typescript_dir / "package.json", package_json)

    tsconfig = {
        "compilerOptions": {
            "target": "ES2020",
            "module": "commonjs",
            "lib": ["ES2020"],
            "outDir": "./dist",
            "rootDir": "./src",
            "strict": True,
            "esModuleInterop": True,
            "skipLibCheck": True,
            "forceConsistentCasingInFileNames": True,
            "declaration": True,
            "declarationMap": True,
            "sourceMap": True,
            "resolveJsonModule": True,
            "moduleResolution": "node",
        },
        "include": ["src/**/*"],
        "exclude": ["node_modules", "dist", "src/web/**", "src/generated/web/**"],
    }
    write_generated_json(typescript_dir / "tsconfig.json", tsconfig)

    tsconfig_web = {
        "compilerOptions": {
            "target": "ES2020",
            "module": "ES2020",
            "moduleResolution": "bundler",
            "lib": ["ES2020", "DOM", "DOM.Iterable"],
            "outDir": "./dist/web",
            "rootDir": "./src",
            "strict": True,
            "esModuleInterop": True,
            "skipLibCheck": True,
            "forceConsistentCasingInFileNames": True,
            "declaration": True,
            "declarationMap": True,
            "sourceMap": True,
            "resolveJsonModule": True,
        },
        "include": [
            "src/shared/**/*",
            "src/web/**/*",
            "src/generated/types/**/*",
            "src/generated/web/**/*",
        ],
        "exclude": ["node_modules", "dist"],
    }
    write_generated_json(typescript_dir / "tsconfig.web.json", tsconfig_web)

    tsconfig_typedoc = {
        "extends": "./tsconfig.json",
        "compilerOptions": {
            "module": "ESNext",
            "moduleResolution": "Bundler",
            "lib": ["ES2020", "DOM", "DOM.Iterable"],
        },
        "include": ["src/**/*"],
        "exclude": ["node_modules", "dist"],
    }
    write_generated_json(typescript_dir / "tsconfig.typedoc.json", tsconfig_typedoc)

    cargo_toml = "\n".join([
        f"# {HEADER_COMMENT}",
        "[package]",
        'name = "goud-engine-node"',
        f'version = "{VERSION}" # x-release-please-version',
        'edition = "2021"',
        'license = "MIT"',
        'description = "Node.js native addon for GoudEngine (napi-rs)"',
        "",
        "[lib]",
        'crate-type = ["cdylib"]',
        "",
        "[dependencies]",
        'napi = { version = "3", default-features = false, features = ["napi8"] }',
        'napi-derive = "3"',
        'goud-engine-core = { path = "../../../goud_engine", features = ["rapier2d", "rapier3d"] }',
        "",
        "[build-dependencies]",
        'napi-build = "2"',
        "",
    ])
    write_generated(typescript_dir / "native" / "Cargo.toml", cargo_toml)

    build_rs = "\n".join([
        f"// {HEADER_COMMENT}",
        "extern crate napi_build;",
        "",
        "fn main() {",
        "    napi_build::setup();",
        "}",
        "",
    ])
    write_generated(typescript_dir / "native" / "build.rs", build_rs)

    # Keep workspace cargo builds viable from a fully wiped generated state.
    # This gets replaced by `gen_ts_node.py` later in codegen.sh.
    bootstrap_lib = "\n".join([
        f"// {HEADER_COMMENT}",
        "",
    ])
    write_generated(typescript_dir / "native" / "src" / "lib.rs", bootstrap_lib)

    wasm_package = {
        "name": "goud-engine-core",
        "type": "module",
        "description": "GoudEngine core - internal implementation crate. Use goud-engine for the public API.",
        "version": VERSION,
        "license": "MIT",
        "repository": {
            "type": "git",
            "url": REPO_URL,
        },
        "files": [
            "goud_engine_bg.wasm",
            "goud_engine.js",
            "goud_engine.d.ts",
        ],
        "main": "goud_engine.js",
        "homepage": REPO_URL,
        "types": "goud_engine.d.ts",
        "sideEffects": ["./snippets/*"],
        "keywords": [
            "game-engine",
            "ecs",
            "rendering",
            "ffi",
        ],
        "x-generatedBy": GENERATED_BY,
    }
    write_generated_json(typescript_dir / "wasm" / "package.json", wasm_package)


def gen_swift_scaffolding() -> None:
    swift_dir = SDKS_DIR / "swift"

    package_swift = "\n".join([
        "// swift-tools-version: 5.9",
        f"// {HEADER_COMMENT}",
        "",
        "import Foundation",
        "import PackageDescription",
        "",
        "let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()",
        'let macDefaultLibSearchPath = packageDir.appendingPathComponent("../../target/release").standardizedFileURL.path',
        'let iosDefaultLibSearchPath = packageDir.appendingPathComponent("../../platform/ios/build/simulator").standardizedFileURL.path',
        "",
        'let macLibSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_LIB_DIR"]',
        '    ?? macDefaultLibSearchPath',
        'let iosLibSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_IOS_LIB_DIR"]',
        '    ?? iosDefaultLibSearchPath',
        "",
        "let package = Package(",
        '    name: "GoudEngine",',
        "    platforms: [",
        "        .macOS(.v13),",
        "        .iOS(.v16),",
        "    ],",
        "    products: [",
        '        .library(name: "GoudEngine", targets: ["GoudEngine"]),',
        "    ],",
        "    targets: [",
        "        .systemLibrary(",
        '            name: "CGoudEngine",',
        '            path: "Sources/CGoudEngine"',
        "        ),",
        "        .target(",
        '            name: "GoudEngine",',
        '            dependencies: ["CGoudEngine"],',
        '            path: "Sources/GoudEngine",',
        "            linkerSettings: [",
        '                .linkedLibrary("goud_engine"),',
        '                .unsafeFlags(["-L", macLibSearchPath], .when(platforms: [.macOS])),',
        '                .unsafeFlags(["-L", iosLibSearchPath], .when(platforms: [.iOS])),',
        "            ]",
        "        ),",
        "        .testTarget(",
        '            name: "GoudEngineTests",',
        '            dependencies: ["GoudEngine"],',
        '            path: "Tests/GoudEngineTests",',
        "            linkerSettings: [",
        '                .linkedLibrary("goud_engine"),',
        '                .unsafeFlags(["-L", macLibSearchPath]),',
        "            ]",
        "        ),",
        "    ]",
        ")",
        "",
    ])
    write_generated(swift_dir / "Package.swift", package_swift)


def main() -> None:
    print("Generating SDK package/build scaffolding...")
    gen_csharp_scaffolding()
    gen_python_scaffolding()
    gen_typescript_scaffolding()
    gen_swift_scaffolding()
    print("SDK package/build scaffolding generation complete.")


if __name__ == "__main__":
    main()
