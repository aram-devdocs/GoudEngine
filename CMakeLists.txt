cmake_minimum_required(VERSION 3.15)
project(GoudEngine)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

# Find SDL2
find_package(SDL2 REQUIRED)
include_directories(${SDL2_INCLUDE_DIRS})

# Link OpenGL based on platform
if(APPLE)
    find_library(OpenGL_LIBRARY OpenGL)
    set(OpenGL_LIBRARIES ${OpenGL_LIBRARY})
else()
    find_package(OpenGL REQUIRED)
endif()

# Add subdirectories
add_subdirectory(engine)
add_subdirectory(modules)
add_subdirectory(tests)
add_subdirectory(samples/BasicSample)

# Link libraries to specific targets
target_link_libraries(GraphicsModule PUBLIC ${SDL2_LIBRARIES} ${OpenGL_LIBRARIES})
target_link_libraries(BasicSample PUBLIC GoudEngineCore GraphicsModule ${SDL2_LIBRARIES} ${OpenGL_LIBRARIES})