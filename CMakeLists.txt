cmake_minimum_required(VERSION 3.15)
project(GoudEngine)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

add_subdirectory(engine)
add_subdirectory(modules)
add_subdirectory(tests)
add_subdirectory(samples/BasicSample)
