cmake_minimum_required(VERSION 3.18 FATAL_ERROR)
project(MyCudaNBody LANGUAGES CXX CUDA)

add_library(nbody SHARED nbody.cu)

set_target_properties(nbody PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
)

install(TARGETS nbody
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
