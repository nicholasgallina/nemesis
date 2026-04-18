use std::process::Command;

fn main() {
    let shaders = [
        ("shaders/simple.vert", "shaders/simple.vert.spv"),
        ("shaders/simple.frag", "shaders/simple.frag.spv"),
        ("shaders/atom.vert", "shaders/atom.vert.spv"),
        ("shaders/atom.frag", "shaders/atom.frag.spv"),
        ("shaders/bond.vert", "shaders/bond.vert.spv"),
        ("shaders/bond.frag", "shaders/bond.frag.spv"),
    ];

    for (src, out) in &shaders {
        let status = Command::new("glslc")
            .args([src, "-o", out])
            .status()
            .expect("ERROR: failed to run glslc");

        if !status.success() {
            panic!("ERROR: shader compilation failed: {}", src);
        }

        println!("cargo:rerun-if-changed={}", src);
    }
}
