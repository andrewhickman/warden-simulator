mod codegen;
mod op;
mod tensor;

use std::path::Path;

use anyhow::{Context, Result};
use prost::Message;
use wdn_rl_onnx::ModelProto;

use self::codegen::Generator;

pub fn generate_model(input: &Path, output: &Path) -> Result<()> {
    let model_bytes = fs_err::read(input)?;
    let model =
        ModelProto::decode(model_bytes.as_slice()).context("failed to decode model proto")?;

    let file = Generator::new()
        .generate(&model)
        .context("failed to generate code")?;

    let file = prettyplease::unparse(&file);
    fs_err::write(output, file)?;

    Ok(())
}
