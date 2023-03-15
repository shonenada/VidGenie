use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("failed to compile shader: {0}")]
    CompileError(String),
}

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("failed to link gl program: {0}")]
    LinkError(String),
}
