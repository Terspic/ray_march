use shaderc;
use std::fs;

const SHADERS: &[(&str, shaderc::ShaderKind)] = &[
    ("main.glsl", shaderc::ShaderKind::Compute),
    ("quad.vert", shaderc::ShaderKind::Vertex),
    ("quad.frag", shaderc::ShaderKind::Fragment),
];

fn main() {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut opts = shaderc::CompileOptions::new().unwrap();

    opts.set_include_callback(|src, _, _, _| {
        Ok(shaderc::ResolvedInclude {
            resolved_name: format!("./assets/shaders/{}", src),
            content: fs::read_to_string(format!("./assets/shaders/{}", src)).unwrap(),
        })
    });
    opts.set_optimization_level(shaderc::OptimizationLevel::Performance);

    for shader in SHADERS {
        let shader_path = format!("./assets/shaders/{}", shader.0);
        let binary_output = compiler
            .compile_into_spirv(
                fs::read_to_string(shader_path).unwrap().as_str(),
                shader.1,
                shader.0,
                "main",
                Some(&opts),
            )
            .unwrap();

        fs::write(
            format!("./assets/compiled_shaders/{}.spv", shader.0),
            binary_output.as_binary_u8(),
        )
        .unwrap()
    }
}
