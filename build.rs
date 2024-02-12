// build.rs

use glob::glob;

fn main() {
    let mut srcs = glob("./LKH-3.0.8/SRC/*.c").unwrap().into_iter().map(|e| e.unwrap().display().to_string()).collect::<Vec<_>>();
    let heads = glob("./LKH-3.0.8/SRC/INCLUDE/*.h").unwrap().into_iter().map(|e| e.unwrap().display().to_string()).collect::<Vec<_>>();
    srcs.push("./src/_elkai.c".to_string());
    cc::Build::new()
        .files(&srcs)
        .include("./LKH-3.0.8/SRC/INCLUDE/")
        .compile("_elkai");

    for path in srcs.iter().chain(heads.iter()) {
        println!("cargo:rerun-if-changed={}", path);
    }
}