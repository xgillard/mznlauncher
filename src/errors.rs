#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("poisoned")]
    Poisoned,
    #[error("io error {0}")]
    Io(#[from] std::io::Error),
    #[error("kill failed {0}")]
    Kill(String),
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::Poisoned
    }
}

impl From<killall::Error> for Error {
    fn from(e: killall::Error) -> Self {
        Self::Kill(format!("{:?}", e))
    }
}
