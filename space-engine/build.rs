use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().expect("workspace root not found");

    // Tell Cargo to rerun this script if the shader source files change.
    //println!(
    //    "cargo:rerun-if-changed={}",
    //    manifest_dir.join("res/shaders/scene/mesh/mesh.vert").display()
    //);
    //println!(
    //    "cargo:rerun-if-changed={}",
    //    manifest_dir.join("res/shaders/scene/mesh/mesh.frag").display()
    //);
    //println!(
    //    "cargo:rerun-if-changed={}",
    //    manifest_dir.join("res/shaders/ui/shader.vert").display()
    //);
    //println!(
    //    "cargo:rerun-if-changed={}",
    //    manifest_dir.join("res/shaders/ui/shader.frag").display()
    //);

    // Create a new shaderc compiler instance.
    // The `unwrap()` is safe here because it will only fail if the native library cannot be loaded.
    let compiler = shaderc::Compiler::new().unwrap();
    // Configure compilation options. This is where you can set optimization levels, etc.
    let mut options = shaderc::CompileOptions::new().unwrap();
    // Set optimization level to performance. Other options like `Size` are also available.
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        shaderc::EnvVersion::Vulkan1_4 as u32,
    );

    // Define the input and output paths.
    // The source shader files are expected in the space-engine crate's res/shaders directory.
    let shader_paths = [
        (
            manifest_dir.join("res/shaders/scene/mesh/mesh.vert"),
            shaderc::ShaderKind::Vertex,
            "scene/mesh/compiled/vert.spv",
        ),
        (
            manifest_dir.join("res/shaders/scene/mesh/mesh.frag"),
            shaderc::ShaderKind::Fragment,
            "scene/mesh/compiled/frag.spv",
        ),
        (
            manifest_dir.join("res/shaders/ui/shader.vert"),
            shaderc::ShaderKind::Vertex,
            "ui/compiled/vert.spv",
        ),
        (
            manifest_dir.join("res/shaders/ui/shader.frag"),
            shaderc::ShaderKind::Fragment,
            "ui/compiled/frag.spv",
        ),
    ];

    let output_dir = workspace_root.join("res/shaders");
    // Create the output directory if it doesn't exist.
    fs::create_dir_all(&output_dir).unwrap();

    for (src_path, kind, dest_file_name) in &shader_paths {
        // Read the shader source code from the file.
        // The `expect()` will provide a clear error message if the file is missing.
        let source_code = fs::read_to_string(src_path).unwrap_or_else(|_| {
            panic!("Failed to read shader source file: {}", src_path.display())
        });

        // Compile the shader source to SPIR-V.
        // The arguments are:
        // - The source code as a string.
        // - The shader kind (Vertex, Fragment, etc.).
        // - The input file name (used for error messages).
        // - The entry point function name (usually "main").
        // - A reference to our compilation options.
        let compilation_result = compiler
            .compile_into_spirv(
                &source_code,
                *kind,
                src_path.to_str().unwrap(),
                "main",
                Some(&options),
            )
            .unwrap_or_else(|_| panic!("Failed to compile shader: {}", src_path.display()));

        // The result contains the compiled binary data.
        let spirv_binary: &[u32] = compilation_result.as_binary();
        // Convert the &[u32] to a &[u8] for writing to a file.
        let spirv_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                spirv_binary.as_ptr() as *const u8,
                std::mem::size_of_val(spirv_binary),
            )
        };

        // Construct the full output path for this shader.
        let output_path = output_dir.join(dest_file_name);
        if let Some(output_parent) = output_path.parent() {
            fs::create_dir_all(output_parent).unwrap_or_else(|_| {
                panic!(
                    "Failed to create output directory: {}",
                    output_parent.display()
                )
            });
        }
        // Write the SPIR-V bytes to the output file.
        fs::write(&output_path, spirv_bytes).unwrap_or_else(|_| {
            panic!(
                "Failed to write compiled shader to: {}",
                output_path.display()
            )
        });

        println!(
            "cargo:warning=Compiled {} to {}",
            src_path.display(),
            output_path.display()
        );
    }
    println!("cargo:warning=Shader compilation complete!");
}
