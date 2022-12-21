use crate::xes::interval::{Event, EventLog, Trace};
use crate::xes::lifecycle;

pub fn lifecycle_to_interval(lifecycle_log: &mut lifecycle::EventLog) -> EventLog {
    let mut event_log = EventLog { traces: vec![] };

    for trace in &mut lifecycle_log.traces {
        let mut new_trace = Trace {
            case: trace.case.clone(),
            variant: trace.variant.clone(),
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
                    activity: pair[0].activity.clone(),
                    resource: pair[0].resource.clone(),
                    start_time: pair[0].timestamp.clone(),
                    end_time: pair[0].timestamp.clone(),
                };

                for event in pair {
                    if event.lifecycle == "start" {
                        new_event.start_time = event.timestamp.clone();
                    } else if event.lifecycle == "complete" {
                        new_event.end_time = event.timestamp.clone();
                    }
                }

                new_trace.events.push(new_event);
            }
        }

        event_log.traces.push(new_trace);
    }

    event_log
}

pub fn interval_to_lifecycle(event_log: &EventLog) -> lifecycle::EventLog {
    let mut lifecycle_log = lifecycle::EventLog { traces: vec![] };

    for trace in &event_log.traces {
        let mut new_trace = lifecycle::Trace {
            case: trace.case.clone(),
            variant: trace.variant.clone(),
            events: Vec::new(),
        };

        for event in &trace.events {
            new_trace.events.push(lifecycle::Event {
                activity: event.activity.clone(),
                resource: event.resource.clone(),
                lifecycle: "start".to_string(),
                timestamp: event.start_time.clone(),
            });
            new_trace.events.push(lifecycle::Event {
                activity: event.activity.clone(),
                resource: event.resource.clone(),
                lifecycle: "complete".to_string(),
                timestamp: event.end_time.clone(),
            });
        }

        lifecycle_log.traces.push(new_trace);
    }

    lifecycle_log
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{conversion, io};
    use crate::xes::{interval, lifecycle};

    fn test_log_path() -> String {
        let mut project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        project_dir.push("event_logs");
        project_dir.push("Production.xes");
        project_dir.to_str().unwrap().to_string()
    }

    #[test]
    fn test_event_log_to_csv() {
        let file_path = test_log_path();

        let mut log = lifecycle::parse_file(&Path::new(&file_path), true);
        let event_log = conversion::lifecycle_to_interval(&mut log);

        assert_eq!(event_log.traces.len(), log.traces.len());
    }

    #[test]
    fn test_interval_to_lifecycle() {
        let data = "case,variant,activity,resource,start_time,end_time\
        \n1,1,A,R1,1,2";
        let mut event_log = interval::EventLog {
            traces: Vec::new(),
        };
        io::csv_to_interval(data.as_bytes(), &mut event_log);

        let lifecycle_log = conversion::interval_to_lifecycle(&event_log);

        assert_eq!(lifecycle_log.traces.len(), 1);
        assert_eq!(lifecycle_log.traces[0].events.len(), 2);
    }
}
