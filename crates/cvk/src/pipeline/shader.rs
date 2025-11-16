use std::path::PathBuf;

use ash::vk;

use utils::{Build, Buildable};
pub use vk::ShaderStageFlags as ShaderStage;

fn to_shader_kind(stage: ShaderStage) -> shaderc::ShaderKind {
    if stage.contains(ShaderStage::VERTEX) {
        shaderc::ShaderKind::Vertex
    } else if stage.contains(ShaderStage::FRAGMENT) {
        shaderc::ShaderKind::Fragment
    } else if stage.contains(ShaderStage::COMPUTE) {
        shaderc::ShaderKind::Compute
    } else if stage.contains(ShaderStage::GEOMETRY) {
        shaderc::ShaderKind::Geometry
    } else if stage.contains(ShaderStage::TESSELLATION_CONTROL) {
        shaderc::ShaderKind::TessControl
    } else if stage.contains(ShaderStage::TESSELLATION_EVALUATION) {
        shaderc::ShaderKind::TessEvaluation
    } else {
        panic!("Unsupported shader stage specified");
    }
}

use crate::Context;

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct Shader {
    handle: vk::ShaderModule,
    stage: ShaderStage,
}

impl Shader {
    #[inline]
    pub const fn stage(&self) -> ShaderStage {
        self.stage
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            Context::get_device().destroy_shader_module(self.handle, None);
        }
    }
}

impl Buildable for Shader {
    type Builder<'a> = ShaderBuilder<'a>;
}

#[derive(Debug, Clone)]
pub enum ShaderCode<'a> {
    FileSPV(PathBuf),
    FileGLSL(PathBuf),
    BufSPV(&'a [u32]),
    StrGLSL(&'a str),
}

#[derive(utils::Paramters, Debug, Clone)]
pub struct ShaderBuilder<'a> {
    stage: ShaderStage,
    code: ShaderCode<'a>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn spv_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.code = ShaderCode::FileSPV(path.into());
        self
    }

    pub fn glsl_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.code = ShaderCode::FileGLSL(path.into());
        self
    }

    pub fn spv_buf(mut self, buf: &'a [u32]) -> Self {
        self.code = ShaderCode::BufSPV(buf);
        self
    }

    pub fn glsl_str(mut self, code: &'a str) -> Self {
        self.code = ShaderCode::StrGLSL(code);
        self
    }
}

impl Default for ShaderBuilder<'_> {
    fn default() -> Self {
        Self {
            stage: ShaderStage::empty(),
            code: ShaderCode::BufSPV(&[]),
        }
    }
}

impl<'a> Build for ShaderBuilder<'a> {
    type Target = Shader;

    fn build(&self) -> Self::Target {
        assert!(
            !self.stage.is_empty(),
            "No shader stage specified in shader builder"
        );

        enum CodeData<'a> {
            GLSL(&'a str),
            SPV(&'a [u32]),
        }

        let spirv_vec;
        let glsl_str;

        let mut file_path = "<internal code>".to_string();

        let code_data = match self.code {
            ShaderCode::FileSPV(ref path_buf) => {
                file_path = path_buf.as_os_str().to_string_lossy().into();

                let data = std::fs::read(path_buf)
                    .expect(&format!("Failed to read shader in file '{}'", file_path));

                spirv_vec = data
                    .chunks_exact(size_of::<u32>())
                    .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
                    .collect::<Vec<u32>>();

                CodeData::SPV(spirv_vec.as_slice())
            }
            ShaderCode::FileGLSL(ref path_buf) => {
                file_path = path_buf.as_os_str().to_string_lossy().into();

                glsl_str = std::fs::read_to_string(path_buf)
                    .expect(&format!("Failed to read shader in file '{}'", file_path));

                CodeData::GLSL(&glsl_str)
            }
            ShaderCode::BufSPV(buf_spv) => CodeData::SPV(buf_spv),
            ShaderCode::StrGLSL(glsl_str) => CodeData::GLSL(glsl_str),
        };

        let compiler_artifact;

        let spv_data = match code_data {
            CodeData::GLSL(glsl_str) => {
                let mut options = shaderc::CompileOptions::new().unwrap();
                options.set_optimization_level(shaderc::OptimizationLevel::Performance);

                let compile_result = Context::get().glsl_compiler().compile_into_spirv(
                    glsl_str,
                    to_shader_kind(self.stage),
                    &file_path,
                    "main",
                    Some(&options),
                );

                compiler_artifact = match compile_result {
                    Ok(value) => value,
                    Err(error) => panic!("Failed to compile GLSL:\n{error}"),
                };

                compiler_artifact.as_binary()
            }
            CodeData::SPV(spv_data) => spv_data,
        };

        let info = vk::ShaderModuleCreateInfo::default().code(spv_data);

        let handle = unsafe { Context::get_device().create_shader_module(&info, None) }
            .expect("Failed to create shader");

        Shader {
            handle,
            stage: self.stage,
        }
    }
}
