use std::io;
use std::sync::mpsc::TryRecvError;

pub enum BioError {
    RecvError(TryRecvError),
    IoError(io::Error),
}

pub type BioResult<T> = Result<T, BioError>;

impl From<TryRecvError> for BioError {
    fn from(e: TryRecvError) -> Self {
        Self::RecvError(e)
    }
}

impl From<io::Error> for BioError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
