use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from("clips-source");
    let helper_dir = PathBuf::from("clips-source-helper");

    let c_files: Vec<PathBuf> = std::fs::read_dir(&src_dir)
        .expect("clips-source directory missing")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|ext| ext == "c").unwrap_or(false))
        .collect();

    if c_files.is_empty() {
        panic!("No .c files found in clips-source/");
    }

    println!("cargo:rerun-if-changed=clips-source/");
    println!("cargo:rerun-if-changed=clips-source-helper/");

    let flags: &[&str] = &[
        "-Wno-implicit-function-declaration",
        "-Wno-unused-result",
        "-Wno-deprecated-declarations",
        "-Wno-unused-parameter",
        "-Wno-sign-compare",
        "-Wno-incompatible-pointer-types-discards-qualifiers",
        "-Wno-int-conversion",
        "-Wno-cast-function-type",
        "-Wno-cast-function-type-mismatch",
        "-Wno-missing-field-initializers",
    ];

    let mut build = cc::Build::new();
    build
        .include(&src_dir)
        // Optimise for speed so benchmarks reflect real-world use
        .opt_level(2);

    for flag in flags {
        build.flag_if_supported(flag);
    }

    for file in &c_files {
        build.file(file);
    }

    // Add the thin C helper shim
    build.file(helper_dir.join("clips_helper.c"));

    build.compile("clips");
}
