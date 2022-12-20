use std::io;
use std::path::Path;

use serde::Serialize;

use crate::xes::interval;
use crate::{conversion, xes};

#[derive(Debug, Clone, Serialize)]
struct Row<'a> {
    case: &'a str,
    variant: &'a str,
    activity: &'a str,
    resource: &'a str,
    start_time: &'a str,
    end_time: &'a str,
}

pub fn interval_to_csv(event_log: &interval::EventLog, writer: &mut impl io::Write) {
    let mut wtr = csv::Writer::from_writer(writer);
    for trace in &event_log.traces {
        for event in &trace.events {
            wtr.serialize(Row {
                case: trace.case,
                variant: trace.variant,
                activity: event.activity,
                resource: event.resource,
                start_time: event.start_time,
                end_time: event.end_time,
            })
            .unwrap();
        }
    }
    wtr.flush().unwrap();
}

pub fn convert_xes_to_csv(input_log: &String, output_dir: &String, filter_start_end_events: bool) {
    let mut log = xes::lifecycle::parse_file(input_log, filter_start_end_events);
    let event_log = conversion::lifecycle_to_interval(&mut log);

    let output_path = Path::new(&output_dir);
    let input_log_path = Path::new(&input_log);
    let output_file_path = output_path
        .join(input_log_path.file_name().unwrap())
        .with_extension("csv");

    let mut csv_file = std::fs::File::create(output_file_path).unwrap();
    interval_to_csv(&event_log, &mut csv_file);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::conversion;
    use crate::xes::lifecycle;

    use super::*;

    fn test_log_path() -> String {
        let mut project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        project_dir.push("event_logs");
        project_dir.push("Production.xes");
        project_dir.to_str().unwrap().to_string()
    }

    #[test]
    fn test_event_log_to_csv() {
        let file_path = test_log_path();

        let mut log = lifecycle::parse_file(&file_path, true);
        let event_log = conversion::lifecycle_to_interval(&mut log);

        let mut csv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        csv_path.push("test_output");
        csv_path.push("Production.csv");
        let csv_file_path = csv_path.to_str().unwrap();

        let mut csv_file = std::fs::File::create(csv_file_path).unwrap();

        interval_to_csv(&event_log, &mut csv_file);
    }
}
