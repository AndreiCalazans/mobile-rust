import org.gradle.api.tasks.Exec

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.example.mobilerust"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.example.mobilerust"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "0.1"
    }

    buildFeatures { compose = true }
    composeOptions { kotlinCompilerExtensionVersion = "1.5.14" }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }

    // The Rust build drops generated Kotlin here; add it as a source set.
    sourceSets["main"].java.srcDir("build/generated/uniffi")
}

// Build the Rust core + regenerate the (gitignored) Kotlin bindings before
// Kotlin compilation. See ../../rust/build-android.sh.
val buildRustCore by tasks.registering(Exec::class) {
    workingDir = rootDir.parentFile.resolve("rust")
    commandLine("./build-android.sh")
}
tasks.matching { it.name.startsWith("compile") && it.name.contains("Kotlin") }
    .configureEach { dependsOn(buildRustCore) }

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.06.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.activity:activity-compose:1.9.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.2")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    // UniFFI generated Kotlin uses JNA to call the .so.
    implementation("net.java.dev.jna:jna:5.14.0@aar")
}
