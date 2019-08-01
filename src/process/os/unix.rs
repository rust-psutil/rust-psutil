use libc;

use crate::process::Process;
use crate::Count;

pub type Uid = libc::uid_t;

pub type Gid = libc::gid_t;

pub trait ProcessExt {
    fn uids(&self) -> Vec<Uid>;

    fn gids(&self) -> Vec<Gid>;

    fn terminal(&self) -> Option<String>;

    fn num_fds(&self) -> Count;
}

impl ProcessExt for Process {
    fn uids(&self) -> Vec<Uid> {
        todo!()
    }

    fn gids(&self) -> Vec<Gid> {
        todo!()
    }

    fn terminal(&self) -> Option<String> {
        todo!()
    }

    fn num_fds(&self) -> Count {
        todo!()
    }
}
