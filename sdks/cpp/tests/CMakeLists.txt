cmake_minimum_required(VERSION 3.14)
project(goud_cpp_tests LANGUAGES CXX)

include(FetchContent)
FetchContent_Declare(
    Catch2
    GIT_REPOSITORY https://github.com/catchorg/Catch2.git
    GIT_TAG        v3.5.2
)
FetchContent_MakeAvailable(Catch2)

# Repo root for finding headers and native lib
get_filename_component(GOUD_REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)

# Find the native library (release preferred, fallback to debug)
find_library(GOUD_NATIVE_LIB goud_engine
    PATHS "${GOUD_REPO_ROOT}/target/release" "${GOUD_REPO_ROOT}/target/debug"
    NO_DEFAULT_PATH
)

# Unified include dir (same approach as FindGoudEngine.cmake for the ../goud_engine.h issue)
set(_test_unified_dir "${CMAKE_CURRENT_BINARY_DIR}/_goud_test_include")
file(MAKE_DIRECTORY "${_test_unified_dir}")
file(COPY "${GOUD_REPO_ROOT}/sdks/c/include/goud" DESTINATION "${_test_unified_dir}")
file(COPY "${GOUD_REPO_ROOT}/codegen/generated/goud_engine.h" DESTINATION "${_test_unified_dir}")

add_executable(goud_cpp_tests
    test_error.cpp
    test_engine_config.cpp
    test_context.cpp
    test_engine.cpp
    test_constants.cpp
)

target_link_libraries(goud_cpp_tests PRIVATE
    Catch2::Catch2WithMain
    ${GOUD_NATIVE_LIB}
)

target_include_directories(goud_cpp_tests PRIVATE
    "${_test_unified_dir}"
    "${GOUD_REPO_ROOT}/sdks/cpp/include"
    "${GOUD_REPO_ROOT}/codegen/generated"
    "${GOUD_REPO_ROOT}/examples/cpp/flappy_bird/src"
)

target_compile_features(goud_cpp_tests PRIVATE cxx_std_17)

include(CTest)
list(APPEND CMAKE_MODULE_PATH ${catch2_SOURCE_DIR}/extras)
include(Catch)
catch_discover_tests(goud_cpp_tests
    PROPERTIES LABELS "cpp_sdk"
)

# Register GL-required tests separately for labeling
# The tests themselves use Catch2 tags [gl_required] for GL-dependent tests
