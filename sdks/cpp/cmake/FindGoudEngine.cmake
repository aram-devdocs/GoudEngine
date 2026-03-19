# FindGoudEngine.cmake
#
# Find the GoudEngine native library and SDK headers.
#
# Inputs:
#   GOUD_ENGINE_ROOT  - (env or CMake variable) path to the GoudEngine repo root
#
# Outputs:
#   GoudEngine_FOUND        - TRUE if headers and library were found
#   GoudEngine_INCLUDE_DIRS - include directories for C, C++, and generated headers
#   GoudEngine_LIBRARIES    - the native library to link against
#
# Imported target:
#   GoudEngine::GoudEngine  - SHARED IMPORTED library target

include(FindPackageHandleStandardArgs)

# ---------------------------------------------------------------------------
# Determine the repository root
# ---------------------------------------------------------------------------

if(DEFINED GOUD_ENGINE_ROOT)
    set(_goud_root "${GOUD_ENGINE_ROOT}")
elseif(DEFINED ENV{GOUD_ENGINE_ROOT})
    set(_goud_root "$ENV{GOUD_ENGINE_ROOT}")
else()
    # Fallback: assume this file lives at <repo>/sdks/cpp/cmake/
    get_filename_component(_goud_root "${CMAKE_CURRENT_LIST_DIR}/../../.." ABSOLUTE)
endif()

# ---------------------------------------------------------------------------
# Find headers
# ---------------------------------------------------------------------------

find_path(GoudEngine_C_INCLUDE_DIR
    NAMES goud/goud.h
    HINTS "${_goud_root}/sdks/c/include"
    NO_DEFAULT_PATH
)

find_path(GoudEngine_CPP_INCLUDE_DIR
    NAMES goud/goud.hpp
    HINTS "${_goud_root}/sdks/cpp/include"
    NO_DEFAULT_PATH
)

find_path(GoudEngine_GENERATED_INCLUDE_DIR
    NAMES goud_engine.h
    HINTS "${_goud_root}/codegen/generated"
    NO_DEFAULT_PATH
)

# ---------------------------------------------------------------------------
# Find the native library
# ---------------------------------------------------------------------------

# Determine platform-specific library name
if(WIN32)
    set(_goud_lib_names goud_engine)
    set(_goud_lib_suffixes ".dll" ".lib")
elseif(APPLE)
    set(_goud_lib_names goud_engine libgoud_engine)
    set(_goud_lib_suffixes ".dylib")
else()
    set(_goud_lib_names goud_engine libgoud_engine)
    set(_goud_lib_suffixes ".so")
endif()

# Search release first, then debug
find_library(GoudEngine_LIBRARY
    NAMES ${_goud_lib_names}
    HINTS
        "${_goud_root}/target/release"
        "${_goud_root}/target/debug"
    NO_DEFAULT_PATH
)

# ---------------------------------------------------------------------------
# Validate
# ---------------------------------------------------------------------------

find_package_handle_standard_args(GoudEngine
    REQUIRED_VARS
        GoudEngine_LIBRARY
        GoudEngine_C_INCLUDE_DIR
        GoudEngine_CPP_INCLUDE_DIR
        GoudEngine_GENERATED_INCLUDE_DIR
)

# ---------------------------------------------------------------------------
# Unified include directory
# ---------------------------------------------------------------------------
#
# The C SDK header (goud/goud.h) uses  #include "../goud_engine.h"  which
# the compiler resolves relative to the physical file location. The
# generated header lives at codegen/generated/goud_engine.h, not at
# sdks/c/include/goud_engine.h where the relative path expects it.
#
# We build a unified include tree in the build directory by *copying* the
# generated header next to the goud/ directory so that the relative include
# resolves correctly. Copies are used instead of symlinks because compilers
# resolve symlinks to the real path before applying relative includes.
# ---------------------------------------------------------------------------

if(GoudEngine_FOUND)
    set(_goud_unified_dir "${CMAKE_CURRENT_BINARY_DIR}/_goud_unified_include")

    # Copy the goud/ headers directory
    file(MAKE_DIRECTORY "${_goud_unified_dir}")
    file(
        COPY "${GoudEngine_C_INCLUDE_DIR}/goud"
        DESTINATION "${_goud_unified_dir}"
    )

    # Copy the generated header alongside goud/ so ../goud_engine.h resolves
    file(
        COPY "${GoudEngine_GENERATED_INCLUDE_DIR}/goud_engine.h"
        DESTINATION "${_goud_unified_dir}"
    )

    # -----------------------------------------------------------------------
    # Exported variables
    # -----------------------------------------------------------------------

    set(GoudEngine_INCLUDE_DIRS
        "${_goud_unified_dir}"
        "${GoudEngine_CPP_INCLUDE_DIR}"
        "${GoudEngine_GENERATED_INCLUDE_DIR}"
    )
    set(GoudEngine_LIBRARIES "${GoudEngine_LIBRARY}")

    # -----------------------------------------------------------------------
    # Imported target
    # -----------------------------------------------------------------------
    if(NOT TARGET GoudEngine::GoudEngine)
        add_library(GoudEngine::GoudEngine SHARED IMPORTED)

        set_target_properties(GoudEngine::GoudEngine PROPERTIES
            IMPORTED_LOCATION "${GoudEngine_LIBRARY}"
            INTERFACE_INCLUDE_DIRECTORIES "${GoudEngine_INCLUDE_DIRS}"
        )

        # On Windows the .dll is the runtime artifact; the .lib is the
        # import library. CMake needs IMPORTED_IMPLIB for the linker.
        if(WIN32)
            get_filename_component(_goud_lib_dir "${GoudEngine_LIBRARY}" DIRECTORY)
            find_file(_goud_implib
                NAMES goud_engine.lib
                HINTS "${_goud_lib_dir}"
                NO_DEFAULT_PATH
            )
            if(_goud_implib)
                set_target_properties(GoudEngine::GoudEngine PROPERTIES
                    IMPORTED_IMPLIB "${_goud_implib}"
                )
            endif()
        endif()
    endif()
endif()

mark_as_advanced(
    GoudEngine_C_INCLUDE_DIR
    GoudEngine_CPP_INCLUDE_DIR
    GoudEngine_GENERATED_INCLUDE_DIR
    GoudEngine_LIBRARY
)
