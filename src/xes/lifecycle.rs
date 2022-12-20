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
