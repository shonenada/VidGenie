use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("failed to compile shader")]
    CompileError(String),
}

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("failed to link gl program")]
    LinkError(String),
}