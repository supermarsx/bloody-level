set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_PROCESSOR arm64)

set(target arm64-pc-windows-msvc)

# The cmake crate contributes MSVC-style flags for windows-msvc targets.
# clang-cl consumes those flags while still using LLVM for ARM64 codegen.
set(CMAKE_C_COMPILER clang-cl)
set(CMAKE_CXX_COMPILER clang-cl)
set(CMAKE_C_COMPILER_TARGET ${target})
set(CMAKE_CXX_COMPILER_TARGET ${target})

# Keep the ARM64 build on the LLVM path required by llama.cpp. The Windows
# SDK and Visual Studio libraries remain available on the hosted runner for
# clang's MSVC-compatible target and the Rust linker.
set(arch_c_flags "-march=armv8.7-a -fvectorize -ffp-model=fast -fno-finite-math-only")
set(warn_c_flags "-Wno-format -Wno-unused-variable -Wno-unused-function -Wno-gnu-zero-variadic-macro-arguments")

set(CMAKE_C_FLAGS_INIT "${arch_c_flags} ${warn_c_flags}")
set(CMAKE_CXX_FLAGS_INIT "${arch_c_flags} ${warn_c_flags}")
