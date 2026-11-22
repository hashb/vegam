cmake_minimum_required(VERSION 3.15...3.30)
project(${SKBUILD_PROJECT_NAME} LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Python 3.10 REQUIRED COMPONENTS Interpreter Development.Module)
find_package(nanobind CONFIG REQUIRED)

# Core C++ library (no Python dependency).
add_library(vegam STATIC src/vegam.cpp)
target_include_directories(vegam PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
set_target_properties(vegam PROPERTIES POSITION_INDEPENDENT_CODE ON)

# Python extension module.
nanobind_add_module(_core NB_STATIC src/bindings.cpp)
target_link_libraries(_core PRIVATE vegam)

install(TARGETS _core LIBRARY DESTINATION vegam)

# Generate type stubs for the native module and ship them in the wheel.
nanobind_add_stub(
  _core_stub
  MODULE _core
  OUTPUT _core.pyi
  PYTHON_PATH $<TARGET_FILE_DIR:_core>
  DEPENDS _core
)

install(FILES ${CMAKE_CURRENT_BINARY_DIR}/_core.pyi DESTINATION vegam)
