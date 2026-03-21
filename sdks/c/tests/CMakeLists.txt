cmake_minimum_required(VERSION 3.16)
project(goud_c_tests LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

get_filename_component(GOUD_REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)

set(GOUD_C_SDK_DIR "${GOUD_REPO_ROOT}/sdks/c/include")

find_library(GOUD_ENGINE_LIB goud_engine
    PATHS "${GOUD_REPO_ROOT}/target/release"
    REQUIRED)

enable_testing()

# Helper function to add a test executable
function(add_goud_test test_name)
    add_executable(${test_name} ${test_name}.c)
    target_include_directories(${test_name} PRIVATE
        "${GOUD_C_SDK_DIR}"
    )
    target_link_libraries(${test_name} PRIVATE "${GOUD_ENGINE_LIB}")
    add_test(NAME ${test_name} COMMAND ${test_name})
endfunction()

add_goud_test(test_constants)
add_goud_test(test_error)
add_goud_test(test_engine_config)
add_goud_test(test_context)
