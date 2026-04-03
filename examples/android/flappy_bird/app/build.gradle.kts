plugins {
    id("com.android.application")
    kotlin("android")
}

val repoRoot = providers.environmentVariable("GOUD_REPO_ROOT")
    .map { file(it).canonicalFile }
    .orElse(rootProject.projectDir.resolve("../../../..").canonicalFile)
    .get()

android {
    namespace = "com.goudengine.flappybird"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.goudengine.flappybird"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets {
        getByName("main") {
            java.srcDirs(
                "src/main/java",
                repoRoot.resolve("sdks/kotlin/src/main/java"),
                repoRoot.resolve("sdks/kotlin/src/main/kotlin"),
            )
            jniLibs.srcDir("src/main/jniLibs")
            assets.srcDir("src/main/assets")
        }
    }

    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
}
