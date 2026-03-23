"""Kotlin SDK generator package."""

from __future__ import annotations

import shutil
from pathlib import Path

from .helpers import (
    HEADER_COMMENT,
    KOTLIN_OUT,
    JAVA_SRC,
    JAVA_DST,
    ROOT_DIR,
    SDKS_DIR,
    schema,
    write_kotlin,
)
from .enums import gen_enums
from .value_types import gen_value_types
from .components import gen_components
from .builders import gen_builders
from .entity import gen_entity
from .errors import gen_errors
from .tools import gen_game, gen_context
from .physics_ui import gen_physics_ui
from .coroutines import gen_coroutines


def _copy_java_carriers():
    """Copy Java carrier classes from JNI test fixtures to SDK."""
    if not JAVA_SRC.exists():
        print(f"  Warning: Java source dir not found: {JAVA_SRC}")
        return

    JAVA_DST.mkdir(parents=True, exist_ok=True)
    count = 0
    for java_file in sorted(JAVA_SRC.glob("*.java")):
        # Skip test harness
        if java_file.name == "JniSmokeMain.java":
            continue
        # Skip SpriteCmd.java — the Kotlin SDK has a Kotlin version in types/
        if java_file.name == "SpriteCmd.java":
            continue
        dst = JAVA_DST / java_file.name
        content = java_file.read_text()
        # In the SDK tree, SpriteCmd lives in com.goudengine.types (Kotlin).
        # The JNI fixtures put it in com.goudengine.internal, so the test
        # fixtures compile without an import. Add the cross-package import
        # for the SDK copy.
        if "SpriteCmd" in content and "import com.goudengine.types.SpriteCmd" not in content:
            content = content.replace(
                "package com.goudengine.internal;\n",
                "package com.goudengine.internal;\n\nimport com.goudengine.types.SpriteCmd;\n",
            )
        dst.write_text(content)
        count += 1

    print(f"  Copied {count} Java carrier classes to {JAVA_DST.relative_to(ROOT_DIR)}")


def _write_gradle_build():
    """Write build.gradle.kts for the Kotlin SDK module."""
    kotlin_root = SDKS_DIR / "kotlin"
    version = schema.get('version', '0.0.1')

    build_gradle = kotlin_root / "build.gradle.kts"
    content = f"""\
// {HEADER_COMMENT}
plugins {{
    kotlin("jvm") version "1.9.22"
    `maven-publish`
    signing
    id("org.jetbrains.dokka") version "1.9.10"
}}

group = "io.github.aram-devdocs"
version = "{version}" // x-release-please-version

repositories {{
    mavenCentral()
}}

dependencies {{
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    testImplementation(kotlin("test"))
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}}

kotlin {{
    jvmToolchain(17)
}}

tasks.test {{
    useJUnitPlatform()
}}

sourceSets {{
    main {{
        java.srcDirs("src/main/java")
        kotlin.srcDirs("src/main/kotlin")
        resources.srcDir("build/native")
    }}
    test {{
        kotlin.srcDirs("src/test/kotlin")
    }}
}}

val buildNative by tasks.registering(Exec::class) {{
    group = "build"
    description = "Build the native Rust library"
    workingDir = rootDir.resolve("../../")
    commandLine("cargo", "build", "--release", "-p", "goud-engine-core")
    doLast {{
        val nativeDir = layout.buildDirectory.dir("native").get().asFile
        nativeDir.mkdirs()
        val osName = System.getProperty("os.name").lowercase()
        val libName = when {{
            "mac" in osName || "darwin" in osName -> "libgoud_engine.dylib"
            "win" in osName -> "goud_engine.dll"
            else -> "libgoud_engine.so"
        }}
        val src = rootDir.resolve("../../target/release/$libName")
        if (src.exists()) {{
            src.copyTo(nativeDir.resolve(libName), overwrite = true)
        }}
    }}
}}

tasks.named("processResources") {{
    dependsOn(buildNative)
}}

tasks.jar {{
    from("build/native") {{
        into("native/")
    }}
}}

java {{
    withSourcesJar()
    withJavadocJar()
}}

publishing {{
    publications {{
        create<MavenPublication>("mavenKotlin") {{
            groupId = "io.github.aram-devdocs"
            artifactId = "goud-engine-kotlin"
            from(components["java"])
            pom {{
                name.set("GoudEngine Kotlin SDK")
                description.set("Kotlin bindings for the GoudEngine game engine")
                url.set("https://github.com/aram-devdocs/GoudEngine")
                licenses {{
                    license {{
                        name.set("MIT")
                        url.set("https://opensource.org/licenses/MIT")
                    }}
                }}
                scm {{
                    connection.set("scm:git:git://github.com/aram-devdocs/GoudEngine.git")
                    developerConnection.set("scm:git:ssh://github.com/aram-devdocs/GoudEngine.git")
                    url.set("https://github.com/aram-devdocs/GoudEngine")
                }}
                developers {{
                    developer {{
                        name.set("GoudEngine Contributors")
                        url.set("https://github.com/aram-devdocs/GoudEngine/graphs/contributors")
                    }}
                }}
            }}
        }}
    }}
    repositories {{
        maven {{
            name = "CentralPortal"
            url = uri("https://central.sonatype.com/repository/maven-releases/")
            credentials {{
                username = System.getenv("MAVEN_USERNAME") ?: ""
                password = System.getenv("MAVEN_PASSWORD") ?: ""
            }}
        }}
    }}
}}

signing {{
    useGpgCmd()
    isRequired = System.getenv("MAVEN_USERNAME") != null
    sign(publishing.publications["mavenKotlin"])
}}
"""
    build_gradle.parent.mkdir(parents=True, exist_ok=True)
    build_gradle.write_text(content)
    print(f"  Generated: {build_gradle.relative_to(ROOT_DIR)}")

    settings_gradle = kotlin_root / "settings.gradle.kts"
    settings_content = f"""\
// {HEADER_COMMENT}
rootProject.name = "goud-engine-kotlin"
"""
    settings_gradle.write_text(settings_content)
    print(f"  Generated: {settings_gradle.relative_to(ROOT_DIR)}")

    gradle_props = kotlin_root / "gradle.properties"
    gradle_props_content = """\
kotlin.code.style=official
org.gradle.jvmargs=-Xmx1g
"""
    gradle_props.write_text(gradle_props_content)
    print(f"  Generated: {gradle_props.relative_to(ROOT_DIR)}")


def _write_library_loader():
    """Write the GoudEngine.kt native library loader."""
    content = f"""\
// {HEADER_COMMENT}
package com.goudengine.core

object GoudEngine {{
    @Volatile
    private var loaded = false

    @Synchronized
    fun ensureLoaded() {{
        if (!loaded) {{
            System.loadLibrary("goud_engine")
            loaded = true
        }}
    }}
}}
"""
    write_kotlin(KOTLIN_OUT / "core" / "GoudEngine.kt", content)


def generate_all() -> None:
    """Generate the complete Kotlin SDK."""
    print("Generating Kotlin SDK...")
    print()

    print("Step 1: Copying Java carriers...")
    _copy_java_carriers()
    print()

    print("Step 2: Writing Gradle build files...")
    _write_gradle_build()
    print()

    print("Step 3: Writing library loader...")
    _write_library_loader()
    print()

    print("Step 4: Generating enums...")
    gen_enums()
    print()

    print("Step 5: Generating value types...")
    gen_value_types()
    print()

    print("Step 6: Generating entity handle...")
    gen_entity()
    print()

    print("Step 7: Generating error types...")
    gen_errors()
    print()

    print("Step 8: Generating components...")
    gen_components()
    print()

    print("Step 9: Generating builders...")
    gen_builders()
    print()

    print("Step 10: Generating tool classes...")
    gen_game()
    gen_context()
    print()

    print("Step 11: Generating physics, UI, and sub-tools...")
    gen_physics_ui()
    print()

    print("Step 12: Generating coroutine extensions...")
    gen_coroutines()
    print()

    print("Kotlin SDK generation complete!")
