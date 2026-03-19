project(
  'goud-engine-cpp',
  'cpp',
  version : '0.1.0',
  default_options : ['cpp_std=c++17'],
)

cpp = meson.get_compiler('cpp')

# --------------------------------------------------------------------------
# Locate the pre-built native library
# --------------------------------------------------------------------------

goud_native = cpp.find_library(
  'goud_engine',
  required : true,
)

# --------------------------------------------------------------------------
# Include directories
# --------------------------------------------------------------------------

repo_root = meson.current_source_dir() / '..' / '..'

c_sdk_inc = include_directories(repo_root / 'sdks' / 'c' / 'include')
cpp_sdk_inc = include_directories('include')
generated_inc = include_directories(repo_root / 'codegen' / 'generated')

# --------------------------------------------------------------------------
# Public dependency that consumers can use via subproject() or dependency()
# --------------------------------------------------------------------------

goud_engine_dep = declare_dependency(
  include_directories : [c_sdk_inc, cpp_sdk_inc, generated_inc],
  dependencies : [goud_native],
)
