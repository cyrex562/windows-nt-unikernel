use thiserror::Error;

#[derive(Error, Debug)]
pub enum PELoaderError {
    #[error("failed to parse binary")]
    ParseError(#[from] goblin::error::Error),
    #[error("failed to load binary: {0}")]
    LoadError(&'static str),
    #[error("failed to apply relocations: {0}")]
    RelocError(&'static str),
    #[error("failed to resolve imports: {0}")]
    ImportError(&'static str),
    #[error("failed to execute binary: {0}")]
    ExecutionError(&'static str),
}
