# portfile.cmake — vcpkg overlay port for GoudEngine C/C++ SDK
#
# Downloads a pre-built native tarball from GitHub Releases and installs
# headers, the native library, and CMake config files.

vcpkg_check_linkage(ONLY_DYNAMIC_LIBRARY)

# Map vcpkg triplet to the GoudEngine release ID
if(VCPKG_TARGET_TRIPLET STREQUAL "x64-linux")
    set(RID "linux-x64")
    set(LIB_NAME "libgoud_engine.so")
elseif(VCPKG_TARGET_TRIPLET STREQUAL "x64-osx")
    set(RID "osx-x64")
    set(LIB_NAME "libgoud_engine.dylib")
elseif(VCPKG_TARGET_TRIPLET STREQUAL "arm64-osx")
    set(RID "osx-arm64")
    set(LIB_NAME "libgoud_engine.dylib")
elseif(VCPKG_TARGET_TRIPLET STREQUAL "x64-windows")
    set(RID "win-x64")
    set(LIB_NAME "goud_engine.dll")
else()
    message(FATAL_ERROR "Unsupported triplet: ${VCPKG_TARGET_TRIPLET}")
endif()

set(TARBALL_NAME "goud-engine-v${VERSION}-${RID}.tar.gz")
set(DOWNLOAD_URL "https://github.com/aram-devdocs/GoudEngine/releases/download/v${VERSION}/${TARBALL_NAME}")

vcpkg_download_distfile(ARCHIVE
    URLS "${DOWNLOAD_URL}"
    FILENAME "${TARBALL_NAME}"
    SHA512 0  # Updated per release; set to 0 for development
)

vcpkg_extract_source_archive(SOURCE_PATH
    ARCHIVE "${ARCHIVE}"
    NO_REMOVE_ONE_LEVEL
)

# The tarball extracts to goud-engine-v${VERSION}-${RID}/
set(EXTRACTED_DIR "${SOURCE_PATH}/goud-engine-v${VERSION}-${RID}")

# Install headers
file(INSTALL "${EXTRACTED_DIR}/include/" DESTINATION "${CURRENT_PACKAGES_DIR}/include")

# Install native library (release)
file(INSTALL "${EXTRACTED_DIR}/lib/${LIB_NAME}" DESTINATION "${CURRENT_PACKAGES_DIR}/lib")

# Install native library (debug — same binary for now)
if(NOT DEFINED VCPKG_BUILD_TYPE OR VCPKG_BUILD_TYPE STREQUAL "debug")
    file(INSTALL "${EXTRACTED_DIR}/lib/${LIB_NAME}" DESTINATION "${CURRENT_PACKAGES_DIR}/debug/lib")
endif()

# Install CMake config
file(INSTALL "${EXTRACTED_DIR}/cmake/" DESTINATION "${CURRENT_PACKAGES_DIR}/share/${PORT}")

# Install copyright
vcpkg_install_copyright(FILE_LIST "${CURRENT_PORT_DIR}/../../LICENSE")
