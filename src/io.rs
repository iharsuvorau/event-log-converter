use std::io;
use serde::Serialize;

use crate::xes::{interval, lifecycle};

#[derive(Debug, Clone, Serialize)]
struct Row {
    case: String,
    variant: String,
    activity: String,
    resource: String,
    start_time: String,
    end_time: String,
}

pub fn interval_to_csv(event_log: &interval::EventLog, writer: &mut impl io::Write) {
    let mut wtr = csv::Writer::from_writer(writer);
    for trace in &event_log.traces {
        for event in &trace.events {
            wtr.serialize(Row {
                case: trace.case.clone(),
                variant: trace.variant.clone(),
                activity: event.activity.clone(),
                resource: event.resource.clone(),
                start_time: event.start_time.clone(),
                end_time: event.end_time.clone(),
            }).unwrap();
        }
    }
    wtr.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::conversion;

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

        let log = lifecycle::parse_file(&file_path, true);
        let event_log = conversion::lifecycle_to_interval(&log);

        let mut csv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        csv_path.push("test_output");
        csv_path.push("Production.csv");
        let csv_file_path = csv_path.to_str().unwrap();

        let mut csv_file = std::fs::File::create(csv_file_path).unwrap();

        interval_to_csv(&event_log, &mut csv_file);
    }
}
