use crate::xes::interval::{Event, EventLog, Trace};
use crate::xes::lifecycle;

pub fn lifecycle_to_interval(lifecycle_log: &mut lifecycle::EventLog) -> EventLog {
    let mut event_log = EventLog { traces: vec![] };

    for trace in &mut lifecycle_log.traces {
        let mut new_trace = Trace {
            case: &trace.case,
            variant: &trace.variant,
            events: Vec::new(),
        };

        let events = &mut trace.events;
        events.sort_unstable_by_key(|e| e.activity.clone());

        for same_activities in events.group_by(|a, b| a.activity == b.activity) {
            assert_eq!(
                same_activities.len() % 2,
                0,
                "Lifecycle log is not valid. Group {same_activities:?}"
            );

            for pair in same_activities.chunks_exact(2) {
                let mut new_event = Event {
                    activity: &pair[0].activity,
                    resource: &pair[0].resource,
                    start_time: &pair[0].timestamp,
                    end_time: &pair[0].timestamp,
                };

                for event in pair {
                    if event.lifecycle == "start" {
                        new_event.start_time = &event.timestamp;
                    } else if event.lifecycle == "complete" {
                        new_event.end_time = &event.timestamp;
                    }
                }

                new_trace.events.push(new_event);
            }
        }

        event_log.traces.push(new_trace);
    }

    event_log
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
        project_dir.push("Application-to-Approval-Government-Agency.xes");
        project_dir.to_str().unwrap().to_string()
    }

    #[test]
    fn test_event_log_to_csv() {
        let file_path = test_log_path();

        let mut log = lifecycle::parse_file(&file_path, true);
        let _ = conversion::lifecycle_to_interval(&mut log);
    }
}
