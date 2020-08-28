use laminar::ErrorKind as LaminarError;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::sync::mpsc::SendError;
use std::sync::{MutexGuard, PoisonError};

use super::{Message, SocketHandle, WorkerInstructions};

#[derive(Debug)]
pub enum NetworkError {
    NoSocket(SocketHandle),
    NoDefaultSocket,
    InternalError(InternalErrorKind),
    IOError(io::Error),
}

#[derive(Debug)]
pub enum InternalErrorKind {
    MutexLockError,
    SendWorkerInstructionsError(String),
    SendMessageError(String),
    LaminarError(LaminarError),
}

use NetworkError::*;

impl From<LaminarError> for NetworkError {
    fn from(err: LaminarError) -> Self {
        match err {
            LaminarError::IOError(err) => IOError(err),
            _ => InternalError(InternalErrorKind::LaminarError(err)), // todo: remove this branch
        }
    }
}

// impl<T> From<PoisonError<MutexGuard<'_, Sender<T>>>> for NetworkError {
impl<T> From<PoisonError<MutexGuard<'_, T>>> for NetworkError {
    fn from(_: PoisonError<MutexGuard<'_, T>>) -> Self {
        InternalError(InternalErrorKind::MutexLockError)
    }
}

impl From<SendError<WorkerInstructions>> for NetworkError {
    fn from(err: SendError<WorkerInstructions>) -> Self {
        InternalError(InternalErrorKind::SendWorkerInstructionsError(
            err.to_string(),
        ))
    }
}

impl From<SendError<Message>> for NetworkError {
    fn from(err: SendError<Message>) -> Self {
        InternalError(InternalErrorKind::SendMessageError(err.to_string()))
    }
}

impl Display for NetworkError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NoSocket(handle) => write!(
                fmt,
                "No socket is currently bound for the handle {:?}",
                handle
            ),
            NoDefaultSocket => write!(fmt, "No default socket is bound."),
            IOError(e) => write!(fmt, "An IO error occurred: {}", e),
            InternalError(e) => write!(fmt, "An internal error occurred: {}", e),
        }
    }
}

impl Display for InternalErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InternalErrorKind::MutexLockError => write!(fmt, "A lock could not be acquired."),
            InternalErrorKind::SendWorkerInstructionsError(e) => write!(
                fmt,
                "A worker instruction could not be sent to the worker thread ({})",
                e
            ),
            InternalErrorKind::SendMessageError(e) => write!(
                fmt,
                "A message could not be sent to the worker thread ({})",
                e
            ),
            InternalErrorKind::LaminarError(e) => {
                write!(fmt, "An unexpected laminar error occurred ({:?})", e)
            }
        }
    }
}

impl Error for NetworkError {}
impl Error for InternalErrorKind {}
