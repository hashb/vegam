// Compiles the bundled vegam C++ library and its C ABI shim.

fn main() {
    let sources = ["src/vegam.cpp", "rust/cpp/ffi.cpp"];

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .include("include")
        .files(sources)
        .warnings(true)
        .compile("vegam");

    for path in sources.iter().copied().chain(["include"]) {
        println!("cargo:rerun-if-changed={path}");
    }
}
