use std::collections::HashMap;
use std::io;

use quick_xml::events::{BytesDecl, Event as XmlEvent};
use serde::{Deserialize, Serialize};

use crate::xes::{interval, lifecycle};
use crate::xes::interval::{EventLog, Trace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogColumns {
    pub case: String,
    pub variant: String,
    pub activity: String,
    pub resource: String,
    pub start_time: String,
    pub end_time: String,
}

impl EventLogColumns {
    pub fn default_style() -> EventLogColumns {
        EventLogColumns {
            case: "case".to_string(),
            variant: "variant".to_string(),
            activity: "activity".to_string(),
            resource: "resource".to_string(),
            start_time: "start_time".to_string(),
            end_time: "end_time".to_string(),
        }
    }
}

pub fn interval_to_csv(event_log: &interval::EventLog, writer: &mut impl io::Write) {
    let mut wtr = csv::Writer::from_writer(writer);
    for trace in &event_log.traces {
        for event in &trace.events {
            wtr.serialize(EventLogColumns {
                case: trace.case.to_string(),
                variant: trace.variant.to_string(),
                activity: event.activity.to_string(),
                resource: event.resource.to_string(),
                start_time: event.start_time.to_string(),
                end_time: event.end_time.to_string(),
            })
            .unwrap();
        }
    }
    wtr.flush().unwrap();
}

pub fn csv_to_interval(bytes: &[u8], event_log: &mut EventLog) {
    let mut reader = csv::Reader::from_reader(bytes);

    let rows = reader
        .deserialize()
        .map(|row| row.expect("Could not deserialize row"))
        .collect::<Vec<HashMap<String, String>>>();

    let traces = rows
        .group_by(|a, b| a.get(&event_log.columns.case).unwrap() == b.get(&event_log.columns.case).unwrap())
        .map(|group| {
            let events = group
                .iter()
                .map(|row| interval::Event {
                    activity: row.get(&event_log.columns.activity).unwrap().to_string(),
                    resource: row.get(&event_log.columns.resource).unwrap().to_string(),
                    start_time: row.get(&event_log.columns.start_time).unwrap().to_string(),
                    end_time: row.get(&event_log.columns.end_time).unwrap().to_string(),
                })
                .collect::<Vec<_>>();

            Trace {
                case: group[0].get(&event_log.columns.case).unwrap().to_string(),
                variant: group[0].get(&event_log.columns.variant).unwrap().to_string(),
                events,
            }
        })
        .collect::<Vec<_>>();

    event_log.traces = traces;
}

pub fn lifecycle_to_xes(event_log: &lifecycle::EventLog, writer: &mut impl io::Write) {
    let mut xml_writer = quick_xml::Writer::new(writer);

    let xml_declaration = BytesDecl::new("1.0", Some("UTF-8"), None);
    xml_writer.write_event(XmlEvent::Decl(xml_declaration)).unwrap();

    xml_writer
        .create_element("log")
        .write_inner_content(|w| {
            event_log.traces.iter().for_each(|trace| {
                w.create_element("trace")
                    .write_inner_content(|w| {
                        w.create_element("string")
                            .with_attribute(("key", "concept:name"))
                            .with_attribute(("value", trace.case.as_str()))
                            .write_empty()
                            .unwrap();
                        w.create_element("string")
                            .with_attribute(("key", "variant"))
                            .with_attribute(("value", trace.variant.as_str()))
                            .write_empty()
                            .unwrap();
                        trace.events.iter().for_each(|event| {
                            w.create_element("event")
                                .write_inner_content(|w| {
                                    w.create_element("string")
                                        .with_attribute(("key", "concept:name"))
                                        .with_attribute(("value", event.activity.as_str()))
                                        .write_empty()
                                        .unwrap();
                                    w.create_element("string")
                                        .with_attribute(("key", "org:resource"))
                                        .with_attribute(("value", event.resource.as_str()))
                                        .write_empty()
                                        .unwrap();
                                    w.create_element("date")
                                        .with_attribute(("key", "time:timestamp"))
                                        .with_attribute(("value", event.timestamp.as_str()))
                                        .write_empty()
                                        .unwrap();
                                    w.create_element("string")
                                        .with_attribute(("key", "lifecycle:transition"))
                                        .with_attribute(("value", event.lifecycle.as_str()))
                                        .write_empty()
                                        .unwrap();
                                    Ok(())
                                })
                                .unwrap();
                        });
                        Ok(())
                    })
                    .unwrap();
            });
            Ok(())
        })
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::conversion;
    use crate::xes::lifecycle;

    use super::*;

    #[test]
    fn test_lifecycle_to_csv() {
        let mut input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_path.push("event_logs");
        input_path.push("Production.xes");

        let mut log = lifecycle::parse_file(Path::new(&input_path), true);
        let event_log = conversion::lifecycle_to_interval(&mut log);

        let mut output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output_path.push("test_output");
        output_path.push("Production.csv");

        let mut csv_file = std::fs::File::create(output_path.clone()).unwrap();

        interval_to_csv(&event_log, &mut csv_file);

        assert!(output_path.exists());

        std::fs::remove_file(&output_path).unwrap();
    }

    #[test]
    fn test_csv_to_interval() {
        let data = "case,variant,activity,resource,start_time,end_time\
        \n1,1,A,R1,1,2";

        let mut event_log = EventLog {
            traces: Vec::new(),
            columns: EventLogColumns::default_style(),
        };

        csv_to_interval(data.as_bytes(), &mut event_log);

        assert_eq!(event_log.traces.len(), 1);
        assert_eq!(event_log.traces[0].events.len(), 1);
        assert_eq!(event_log.traces[0].case, "1");
        assert_eq!(event_log.traces[0].variant, "1");
        assert_eq!(event_log.traces[0].events[0].activity, "A");
        assert_eq!(event_log.traces[0].events[0].resource, "R1");
        assert_eq!(event_log.traces[0].events[0].start_time, "1");
        assert_eq!(event_log.traces[0].events[0].end_time, "2");
    }

    #[test]
    fn test_csv_to_interval_with_columns() {
        let data = "case_id,variant,Activity,Resource,start_timestamp,end_timestamp\
        \n1,1,A,R1,1,2";

        let mut event_log = EventLog {
            traces: Vec::new(),
            columns: EventLogColumns {
                case: "case_id".to_string(),
                variant: "variant".to_string(),
                activity: "Activity".to_string(),
                resource: "Resource".to_string(),
                start_time: "start_timestamp".to_string(),
                end_time: "end_timestamp".to_string(),
            },
        };

        csv_to_interval(data.as_bytes(), &mut event_log);

        assert_eq!(event_log.traces.len(), 1);
        assert_eq!(event_log.traces[0].events.len(), 1);
        assert_eq!(event_log.traces[0].case, "1");
        assert_eq!(event_log.traces[0].variant, "1");
        assert_eq!(event_log.traces[0].events[0].activity, "A");
        assert_eq!(event_log.traces[0].events[0].resource, "R1");
        assert_eq!(event_log.traces[0].events[0].start_time, "1");
        assert_eq!(event_log.traces[0].events[0].end_time, "2");
    }

    #[test]
    fn test_csv_to_interval_from_file() {
        let mut input_log = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_log.push("event_logs");
        input_log.push("Production_custom.csv");

        let mut event_log = EventLog {
            traces: Vec::new(),
            columns: EventLogColumns::default_style(),
        };
        let bytes = std::fs::read(input_log).unwrap();

        csv_to_interval(&bytes, &mut event_log);

        assert!(event_log.traces.len() > 2);
    }

    #[test]
    fn test_lifecycle_to_xes() {
        let mut input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_path.push("event_logs");
        input_path.push("Production.xes");
        let log = lifecycle::parse_file(&input_path, true);

        let mut xes_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        xes_path.push("test_output");
        xes_path.push("Production.xes");

        let mut xes_file = std::fs::File::create(xes_path.clone()).unwrap();

        lifecycle_to_xes(&log, &mut xes_file);

        assert!(xes_path.exists());

        std::fs::remove_file(&xes_path).unwrap();
    }
}
