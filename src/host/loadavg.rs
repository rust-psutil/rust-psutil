use crate::{Count, FloatCount, Pid};

#[derive(Debug)]
pub struct LoadAvg {
    /// Number of jobs in the run queue averaged over 1 minute.
    pub one: FloatCount,

    /// Number of jobs in the run queue averaged over 5 minute.
    pub five: FloatCount,

    /// Number of jobs in the run queue averaged over 15 minute.
    pub fifteen: FloatCount,

    /// Current number of runnable kernel entities.
    pub runnable: Count,

    /// Total number of runnable kernel entities.
    pub total_runnable: Count,

    /// PID for the most recently created process.
    pub last_pid: Pid,
}
