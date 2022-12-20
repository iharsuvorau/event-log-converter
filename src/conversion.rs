use crate::xes::interval::{Event, EventLog, Trace};
use crate::xes::lifecycle;

pub fn lifecycle_to_interval(lifecycle_log: &lifecycle::EventLog) -> EventLog {
    let mut event_log = EventLog { traces: Vec::new() };

    for trace in &lifecycle_log.traces {
        let mut new_trace = Trace {
            case: trace.case.clone(),
            variant: trace.variant.clone(),
            events: Vec::new(),
        };

        let mut events = trace.events.clone();
        events.sort_by_key(|e| e.activity.clone());

        for same_activities in events.group_by(|a, b| a.activity == b.activity) {
            assert_eq!(
                same_activities.len() % 2,
                0,
                "Lifecycle log is not valid. Group {same_activities:?}"
            );

            for pair in same_activities.chunks(2) {
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

                new_trace.events.sort_by_key(|e| e.start_time.clone());
                new_trace.events.push(new_event);
            }
        }

        event_log.traces.sort_by_key(|t| t.events[0].start_time.clone());
        event_log.traces.push(new_trace);
    }

    event_log
}
