use std::fs;
use std::io;
use std::str::FromStr;

use crate::host::LoadAvg;
use crate::utils::invalid_data;

impl FromStr for LoadAvg {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = s.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(invalid_data(&format!("malformed loadavg data: '{}'", s)));
        }
        let one = try_parse!(fields[0]);
        let five = try_parse!(fields[1]);
        let fifteen = try_parse!(fields[2]);

        let last_pid = try_parse!(fields[4]);

        let entities = fields[3].split('/').collect::<Vec<&str>>();
        if entities.len() != 2 {
            return Err(invalid_data(&format!("malformed loadavg data: '{}'", s)));
        }
        let runnable = try_parse!(entities[0]);
        let total_runnable = try_parse!(entities[1]);

        Ok(LoadAvg {
            one,
            five,
            fifteen,
            runnable,
            total_runnable,
            last_pid,
        })
    }
}

pub fn loadavg() -> io::Result<LoadAvg> {
    let data = fs::read_to_string("/proc/loadavg")?;

    LoadAvg::from_str(&data)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::FloatCount;
    use float_cmp::approx_eq;

    #[test]
    fn test_loadaverage() {
        let loadavg = loadavg().unwrap();
        // shouldn't be negative
        assert!(loadavg.one >= 0.0);
        assert!(loadavg.five >= 0.0);
        assert!(loadavg.fifteen >= 0.0);
    }

    #[test]
    fn test_parse_loadavg() {
        let input = "0.49 0.70 0.84 2/519 1454\n";
        let loadavg = LoadAvg::from_str(input).unwrap();
        assert!(approx_eq!(FloatCount, loadavg.one, 0.49));
        assert!(approx_eq!(FloatCount, loadavg.five, 0.70));
        assert!(approx_eq!(FloatCount, loadavg.fifteen, 0.84));
    }
}
