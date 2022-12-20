use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::xes::lifecycle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub traces: Vec<Trace>,
}

impl EventLog {
    pub fn pretty_print(&self) {
        for trace in &self.traces {
            println!("Trace: {}", trace.case);
            println!("Variant: {}", trace.variant);
            for event in &trace.events {
                println!("  Event: {}", event.activity);
                println!("    Resource: {}", event.resource);
                println!("    Start: {}", event.start_time.format("%Y-%m-%dT%H:%M:%S%z"));
                println!("    End: {}", event.end_time.format("%Y-%m-%dT%H:%M:%S%z"));
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub case: String,
    pub variant: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub activity: String,
    pub resource: String,
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime",
        rename = "start_time"
    )]
    pub start_time: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_datetime",
        deserialize_with = "deserialize_datetime",
        rename = "end_time"
    )]
    pub end_time: DateTime<Utc>,
}

pub fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc))
}

pub fn lifecycle_to_event_log(lifecycle_log: &lifecycle::EventLog) -> EventLog {
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
                    start_time: pair[0].timestamp,
                    end_time: pair[0].timestamp,
                };

                for event in pair {
                    if event.lifecycle == "start" {
                        new_event.start_time = event.timestamp;
                    } else if event.lifecycle == "complete" {
                        new_event.end_time = event.timestamp;
                    }
                }

                new_trace.events.sort_by_key(|e| e.start_time);
                new_trace.events.push(new_event);
            }
        }

        event_log.traces.sort_by_key(|t| t.events[0].start_time);
        event_log.traces.push(new_trace);
    }

    event_log
}
