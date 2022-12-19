#![feature(slice_group_by)]

pub mod xes {
    use chrono::{DateTime, Utc};

    pub mod lifecycle {
        use std::io::Read;

        use chrono::{DateTime, Utc};
        use quick_xml::events::Event as XmlEvent;
        use quick_xml::Reader;

        #[derive(Debug, Clone)]
        pub struct EventLog {
            pub traces: Vec<Trace>,
        }

        #[derive(Debug, Clone)]
        pub struct Trace {
            pub case: String,
            pub variant: String,
            pub events: Vec<Event>,
        }

        #[derive(Debug, Clone)]
        pub struct Event {
            pub activity: String,
            pub resource: String,
            pub timestamp: DateTime<Utc>,
            pub lifecycle: String,
        }

        pub fn parse_file(file_name: &str) -> EventLog {
            let xml = std::fs::read_to_string(file_name).unwrap();
            let mut reader = Reader::from_str(&xml);
            reader.trim_text(true);

            let mut event_log = EventLog { traces: Vec::new() };

            let mut buf = Vec::new();
            let mut current_trace = Trace {
                case: String::new(),
                variant: String::new(),
                events: Vec::new(),
            };
            let mut current_event = Event {
                activity: String::new(),
                resource: String::new(),
                timestamp: Utc::now(),
                lifecycle: String::new(),
            };
            let mut in_trace = false;
            let mut in_event = false;
            let mut in_name = false;
            let mut in_resource = false;
            let mut in_timestamp = false;
            let mut in_lifecycle = false;
            let mut in_variant = false;

            loop {
                match reader.expand_empty_elements(false).read_event_into(&mut buf) {
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    Ok(XmlEvent::Eof) => break,

                    Ok(XmlEvent::Start(ref e)) => match e.name().as_ref() {
                        b"trace" => {
                            in_trace = true;
                            current_trace = Trace {
                                case: String::new(),
                                variant: String::new(),
                                events: Vec::new(),
                            };
                        }

                        b"event" => {
                            in_event = true;
                            current_event = Event {
                                activity: String::new(),
                                resource: String::new(),
                                timestamp: Utc::now(),
                                lifecycle: String::new(),
                            };
                        }

                        _ => (),
                    },

                    Ok(XmlEvent::End(ref e)) => match e.name().as_ref() {
                        b"trace" => {
                            in_trace = false;
                            event_log.traces.push(current_trace.clone());
                        }
                        b"event" => {
                            in_event = false;
                            current_trace.events.push(current_event.clone());
                        }
                        _ => (),
                    },

                    Ok(XmlEvent::Empty(ref e)) => match e.name().as_ref() {
                        b"string" | b"date" => {
                            e.attributes().for_each(|a| {
                                let a = a.unwrap();
                                let mut key = String::new();
                                let mut value = String::new();
                                let _ = a.key.as_ref().read_to_string(&mut key);
                                let _ = a.value.as_ref().read_to_string(&mut value);

                                if key == "key" {
                                    match value.as_str() {
                                        "concept:name" => {
                                            in_name = true;
                                            in_resource = false;
                                            in_timestamp = false;
                                            in_lifecycle = false;
                                            in_variant = false;
                                        }
                                        "org:resource" => {
                                            in_name = false;
                                            in_resource = true;
                                            in_timestamp = false;
                                            in_lifecycle = false;
                                            in_variant = false;
                                        }
                                        "time:timestamp" => {
                                            in_name = false;
                                            in_resource = false;
                                            in_timestamp = true;
                                            in_lifecycle = false;
                                            in_variant = false;
                                        }
                                        "lifecycle:transition" => {
                                            in_name = false;
                                            in_resource = false;
                                            in_timestamp = false;
                                            in_lifecycle = true;
                                            in_variant = false;
                                        }
                                        "variant" => {
                                            in_name = false;
                                            in_resource = false;
                                            in_timestamp = false;
                                            in_lifecycle = false;
                                            in_variant = true;
                                        }
                                        _ => {
                                            in_name = false;
                                            in_resource = false;
                                            in_timestamp = false;
                                            in_lifecycle = false;
                                            in_variant = false;
                                        }
                                    }
                                }

                                if key == "value" {
                                    if in_name {
                                        if in_event {
                                            current_event.activity = value;
                                        } else if in_trace {
                                            current_trace.case = value;
                                        }
                                    } else if in_resource && in_event {
                                        current_event.resource = value;
                                    } else if in_timestamp && in_event {
                                        current_event.timestamp =
                                            DateTime::parse_from_rfc3339(&value).unwrap().with_timezone(&Utc);
                                    } else if in_lifecycle && in_event {
                                        current_event.lifecycle = value;
                                    } else if in_variant && in_trace {
                                        current_trace.variant = value;
                                    }
                                }
                            });
                        }

                        _ => (),
                    },

                    _ => (),
                }

                buf.clear();
            }

            event_log
        }
    }

    #[derive(Debug, Clone)]
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

    #[derive(Debug, Clone)]
    pub struct Trace {
        pub case: String,
        pub variant: String,
        pub events: Vec<Event>,
    }

    #[derive(Debug, Clone)]
    pub struct Event {
        pub activity: String,
        pub resource: String,
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xes() {
        let file_path = "";
        let log = xes::lifecycle::parse_file(file_path);
        println!("{log:?}");
    }

    #[test]
    fn test_lifecycle_to_event_log() {
        let file_path = "";
        let log = xes::lifecycle::parse_file(file_path);
        let event_log = xes::lifecycle_to_event_log(&log);
        // println!("{event_log:?}");
        event_log.pretty_print();
    }
}
