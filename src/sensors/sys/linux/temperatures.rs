use std::collections::HashMap;
use std::io;

use crate::Temperature;

pub struct TemperatureReading {
    _label: Option<String>,
    _current: Temperature,
    _max: Temperature,
    _crit: Temperature,
    _crit_alarm: Temperature,
}

pub fn temperatures() -> io::Result<HashMap<String, Vec<TemperatureReading>>> {
    todo!()
}
