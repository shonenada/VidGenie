use thiserror::Error;

#[derive(Display, Debug, Error)]
pub enum ShaderError {
    CompileError(String),
}

#[derive(Display, Debug, Error)]
pub enum ProgramError {
    LinkError(String),
}