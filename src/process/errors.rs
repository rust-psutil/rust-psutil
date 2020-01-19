use std::io;

use nix;
use snafu::Snafu;

use crate::Pid;

pub type ProcessResult<T> = std::result::Result<T, ProcessError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ProcessError {
    #[snafu(display("Process {} does not exists", pid))]
    NoSuchProcess { pid: Pid },

    #[snafu(display("Process {} is a zombie", pid))]
    ZombieProcess { pid: Pid },

    #[snafu(display("Access denied for process {}", pid))]
    AccessDenied { pid: Pid },

    #[snafu(display("io error for process {}: {}", pid, source))]
    IoError { pid: Pid, source: io::Error },

    #[snafu(display("nix error for process {}: {}", pid, source))]
    NixError { pid: Pid, source: nix::Error },
}

pub fn io_error_to_process_error(e: io::Error, pid: Pid) -> ProcessError {
    match e.kind() {
        io::ErrorKind::NotFound => ProcessError::NoSuchProcess { pid },
        io::ErrorKind::PermissionDenied => ProcessError::AccessDenied { pid },
        _ => ProcessError::IoError { pid, source: e },
    }
}
