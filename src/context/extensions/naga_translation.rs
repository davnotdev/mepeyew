use super::*;
use naga::{back, front, valid, ShaderStage};

///  Additional information to pass to naga.
///  This is currently empty.
#[derive(Default)]
pub struct NagaTranslationExtensionTranslateShaderCodeExt {}

///  The type of shader to be translated.
pub enum NagaTranslationStage {
    Vertex,
    Fragment,
    Compute,
}

///  The expected input language.
pub enum NagaTranslationInput {
    Glsl,
    Spirv,
    Wgsl,
}

enum NagaTranslationOutput {
    Spirv,
    Wgsl,
}

///  Translate shader code into the language accepted by the current rendering backend.
///  `code` should be in the a byte slice obtained with `"...".as_bytes()`.
impl Context {
    pub fn naga_translation_extension_translate_shader_code(
        &self,
        stage: NagaTranslationStage,
        input: NagaTranslationInput,
        code: &[u8],
        _ext: NagaTranslationExtensionTranslateShaderCodeExt,
    ) -> GResult<Vec<u8>> {
        self.assert_extension_enabled(ExtensionType::NagaTranslation);

        let output = match self {
            Context::Vulkan(_) => NagaTranslationOutput::Spirv,
            Context::WebGpu(_) => NagaTranslationOutput::Wgsl,
        };

        let stage = match stage {
            NagaTranslationStage::Vertex => ShaderStage::Vertex,
            NagaTranslationStage::Fragment => ShaderStage::Fragment,
            NagaTranslationStage::Compute => ShaderStage::Compute,
        };

        let in_module = match input {
            NagaTranslationInput::Glsl => {
                let mut parser = front::glsl::Frontend::default();
                let options = front::glsl::Options::from(stage);
                parser
                    .parse(
                        &options,
                        std::str::from_utf8(code)
                            .map_err(|e| gpu_api_err!("naga glsl in slice->str: {:?}", e))?,
                    )
                    .map_err(|e| gpu_api_err!("naga glsl in parse: {:?}", e))?
            }
            NagaTranslationInput::Wgsl => front::wgsl::parse_str(
                std::str::from_utf8(code)
                    .map_err(|e| gpu_api_err!("naga wgsl in slice->str: {:?}", e))?,
            )
            .map_err(|e| gpu_api_err!("naga wgsl in parse: {:?}", e))?,
            NagaTranslationInput::Spirv => {
                let options = front::spv::Options {
                    adjust_coordinate_space: false,
                    strict_capabilities: false,
                    ..Default::default()
                };
                front::spv::parse_u8_slice(code, &options)
                    .map_err(|e| gpu_api_err!("naga spriv in parse: {:?}", e))?
            }
        };

        let info = valid::Validator::new(
            valid::ValidationFlags::all(),
            valid::Capabilities::default(),
        )
        .validate(&in_module)
        .map_err(|e| gpu_api_err!("naga validation: {:?}", e))?;

        Ok(match output {
            NagaTranslationOutput::Spirv => {
                let pipeline_options = back::spv::PipelineOptions {
                    entry_point: "main".to_owned(),
                    shader_stage: stage,
                };

                let flags = back::spv::WriterFlags::ADJUST_COORDINATE_SPACE;
                let options = back::spv::Options {
                    flags,
                    ..Default::default()
                };

                let vec =
                    back::spv::write_vec(&in_module, &info, &options, Some(&pipeline_options))
                        .map_err(|e| gpu_api_err!("naga spirv out: {:?}", e))?;
                vec.iter()
                    .fold(Vec::with_capacity(vec.len() * 4), |mut v, w| {
                        v.extend_from_slice(&w.to_le_bytes());
                        v
                    })
            }
            NagaTranslationOutput::Wgsl => {
                back::wgsl::write_string(&in_module, &info, back::wgsl::WriterFlags::empty())
                    .map_err(|e| gpu_api_err!("naga spirv out: {:?}", e))?
                    .into_bytes()
            }
        })
    }
}
