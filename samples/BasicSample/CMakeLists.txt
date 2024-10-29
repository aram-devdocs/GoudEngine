project(BasicSample)

include_directories(
    ${CMAKE_SOURCE_DIR}/engine/include  
    ${CMAKE_SOURCE_DIR}/modules/graphics/include  
)

add_executable(BasicSample main.cpp)

target_link_libraries(BasicSample PRIVATE GoudEngineCore GraphicsModule)