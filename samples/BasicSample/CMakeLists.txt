project(BasicSample)

include_directories(
    ${CMAKE_SOURCE_DIR}/modules/engine/include  
    ${CMAKE_SOURCE_DIR}/modules/submodules/graphics/include  
)

add_executable(BasicSample main.cpp)

target_link_libraries(BasicSample PRIVATE EngineModule GraphicsModule)