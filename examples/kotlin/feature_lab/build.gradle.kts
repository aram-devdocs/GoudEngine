plugins {
    kotlin("jvm") version "1.9.22"
    application
}

group = "com.goudengine.examples"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation(fileTree("../../../sdks/kotlin/build/libs") { include("*.jar") })
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
}

kotlin {
    jvmToolchain(17)
}

application {
    mainClass.set("MainKt")
}
