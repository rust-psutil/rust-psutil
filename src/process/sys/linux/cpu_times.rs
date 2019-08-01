use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ProcessCpuTimes {
    pub(crate) user: Duration,
    pub(crate) system: Duration,
    pub(crate) children_user: Duration,
    pub(crate) children_system: Duration,
    pub(crate) iowait: Duration,
}

impl ProcessCpuTimes {
    pub fn user(&self) -> Duration {
        self.user
    }

    pub fn system(&self) -> Duration {
        self.system
    }

    pub fn children_user(&self) -> Duration {
        self.children_user
    }

    pub fn children_system(&self) -> Duration {
        self.children_system
    }
}
