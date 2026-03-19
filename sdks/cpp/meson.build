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
# Unified include directory
# --------------------------------------------------------------------------
#
# The C SDK header (goud/goud.h) uses  #include "../goud_engine.h"  which
# resolves relative to the physical file location. We copy the C SDK headers
# and the generated header into a single build-tree directory so the relative
# include resolves correctly — same approach as FindGoudEngine.cmake.
# --------------------------------------------------------------------------

repo_root = meson.current_source_dir() / '..' / '..'
c_sdk_goud_dir = repo_root / 'sdks' / 'c' / 'include' / 'goud'
generated_header = repo_root / 'codegen' / 'generated' / 'goud_engine.h'

# Build a unified include tree at configure time.
unified_dir = meson.current_build_dir() / '_goud_unified_include'

run_command('mkdir', '-p', unified_dir / 'goud', check : true)
run_command('sh', '-c',
  'cp "' + c_sdk_goud_dir + '"/*.h "' + unified_dir + '/goud/"',
  check : true,
)
run_command('cp', generated_header, unified_dir / 'goud_engine.h',
  check : true,
)

unified_inc = include_directories(unified_dir, is_system : false)
cpp_sdk_inc = include_directories('include')
generated_inc = include_directories(repo_root / 'codegen' / 'generated')

# --------------------------------------------------------------------------
# Public dependency that consumers can use via subproject() or dependency()
# --------------------------------------------------------------------------

goud_engine_dep = declare_dependency(
  include_directories : [unified_inc, cpp_sdk_inc, generated_inc],
  dependencies : [goud_native],
)
