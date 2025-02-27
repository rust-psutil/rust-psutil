use psutil::*;
use std::collections::{BTreeMap, HashMap};

fn main() {
	let temperatures = sensors::temperatures();

	// Dictionaries to store sensor data categorized by type
	let mut acpi_sensors = HashMap::new();
	let mut cpu_sensors = BTreeMap::new();
	let mut disk_sensors = HashMap::new();

	// Iterate over all temperature sensors
	temperatures.iter().for_each(|sensor| {
        if let Ok(temp_sensor) = sensor {
            // Extract sensor id from hwmon
            let sensor_id = temp_sensor.hwmon_id().unwrap_or("Unknown").to_string();

            // Categorize the sensor by type (acpitz, nvme, coretemp)
            match temp_sensor.unit() {
                "acpitz" => {
                    acpi_sensors.entry(sensor_id.clone()).or_insert(Vec::new());
                    let msg = format!("Chipset: {:>3}°C", temp_sensor.current().celsius());

                    acpi_sensors
                        .get_mut(&sensor_id)
                        .unwrap()
                        .push(HashMap::from([("acpitz", msg)]));
                }
                "nvme" => {
                    disk_sensors.entry(sensor_id.clone()).or_insert(Vec::new());
                    let msg = format!(
                        "NVME Disk: {:>3}°C Type: {:<9}  (Max = +{}°C, Critical = +{}°C, Min = {}°C)",
                        temp_sensor.current().celsius(),
                        temp_sensor.label().unwrap_or("Unknown"),
                        temp_sensor.high().unwrap_or(&Temperature::new(0.0)).celsius(),
                        temp_sensor.critical().unwrap_or(&Temperature::new(0.0)).celsius(),
                        temp_sensor.min().unwrap_or(&Temperature::new(0.0)).celsius()
                    );

                    disk_sensors
                        .get_mut(&sensor_id)
                        .unwrap()
                        .push(HashMap::from([(temp_sensor.label().unwrap(), msg)]));
                }
                "coretemp" => {
                    cpu_sensors.entry(sensor_id.clone()).or_insert(Vec::new());
                    if let Some(label) = temp_sensor.label() {
                        let core_num = label.split_whitespace().last().unwrap();
                        if label.to_lowercase().contains("package") {
                            let msg = format!(
                                "Package {:>2}: {:>3}°C (Max = +{}°C, Critical = +{}°C)",
                                core_num,
                                temp_sensor.current().celsius(),
                                temp_sensor.high().unwrap_or(&Temperature::new(0.0)).celsius(),
                                temp_sensor.critical().unwrap_or(&Temperature::new(0.0)).celsius()
                            );
                            cpu_sensors
                                .get_mut(&sensor_id)
                                .unwrap()
                                .push(HashMap::from([(temp_sensor.label().unwrap(), msg)]));
                        } else {
                            let msg = format!(
                                "Core {:>2}: {:>6}°C (Max = +{}°C, Critical = +{}°C)",
                                core_num,
                                temp_sensor.current().celsius(),
                                temp_sensor.high().unwrap_or(&Temperature::new(0.0)).celsius(),
                                temp_sensor.critical().unwrap_or(&Temperature::new(0.0)).celsius()
                            );
                            cpu_sensors
                                .get_mut(&sensor_id)
                                .unwrap()
                                .push(HashMap::from([(temp_sensor.label().unwrap(), msg)]));
                        }
                    }
                }
                _ => {
                    // Unknown sensor type
                }
            };
        }
    });

	// Output ACPI sensor data if available
	if !acpi_sensors.is_empty() {
		acpi_sensors.iter_mut().for_each(|(_sensor_id, values)| {
			for value in values {
				value.iter().for_each(|(_key, msg)| {
					println!("{}", msg);
				});
			}
		});
	}

	// Output CPU sensor data if available
	if !cpu_sensors.is_empty() {
		println!();
		cpu_sensors.iter_mut().for_each(|(_sensor_id, values)| {
			// Sort CPU cores by the number part of "Core X" (e.g., Core 0, Core 1)
			values.sort_by_key(|map| {
				let key = map.keys().next().unwrap(); // Get the key from the HashMap
				if key.contains("Package") {
					return 0; // Make sure Package is at the front
				}
				key.split_whitespace()
					.nth(1)
					.and_then(|num| num.parse::<u32>().ok()) // Parse the number
					.map(|n| n + 1) // Ensure "Package id 0" (0) comes first
					.unwrap_or(u32::MAX) // If parsing fails, place it at the end
			});

			for value in values {
				value.iter().for_each(|(_key, msg)| {
					println!("{}", msg);
				});
			}
		});
	}

	// Output Disk sensor data if available
	if !disk_sensors.is_empty() {
		println!();
		disk_sensors.iter_mut().for_each(|(_sensor_id, values)| {
			for value in values {
				let (_key, msg) = value.iter().next().unwrap();
				println!("{}", msg);
			}
		});
	}
}
