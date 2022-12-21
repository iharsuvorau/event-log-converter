use std::io;
use std::io::BufWriter;
use std::path::Path;

use quick_xml::events::{BytesDecl, Event as XmlEvent};
use serde::{Deserialize, Serialize};

use crate::conversion;
use crate::xes::interval::{EventLog, Trace};
use crate::xes::{interval, lifecycle};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        .collect::<Vec<Row>>();

    let traces = rows
        .group_by(|a, b| a.case == b.case)
        .map(|group| {
            let events = group
                .iter()
                .map(|row| interval::Event {
                    activity: row.activity.clone(),
                    resource: row.resource.clone(),
                    start_time: row.start_time.clone(),
                    end_time: row.end_time.clone(),
                })
                .collect::<Vec<_>>();

            Trace {
                case: group[0].case.clone(),
                variant: group[0].variant.clone(),
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

pub fn convert_xes_to_csv(input_log: &Path, output_dir: &Path, filter_start_end_events: bool) {
    let mut log = lifecycle::parse_file(input_log, filter_start_end_events);
    let event_log = conversion::lifecycle_to_interval(&mut log);

    let input_log_path = Path::new(&input_log);
    let output_file_path = output_dir
        .join(input_log_path.file_name().unwrap())
        .with_extension("csv");

    let mut csv_file = std::fs::File::create(output_file_path).unwrap();
    interval_to_csv(&event_log, &mut csv_file);
}

pub fn convert_csv_to_xes(input_log: &Path, output_dir: &Path) {
    let mut event_log = EventLog { traces: Vec::new() };
    let bytes = std::fs::read(input_log).unwrap();
    csv_to_interval(&bytes, &mut event_log);

    let interval_log = conversion::interval_to_lifecycle(&event_log);

    let output_file_path = output_dir.join(input_log.file_name().unwrap()).with_extension("xes");

    let mut xes_file = BufWriter::new(std::fs::File::create(output_file_path).unwrap());
    lifecycle_to_xes(&interval_log, &mut xes_file);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::conversion;
    use crate::xes::lifecycle;

    use super::*;

    #[test]
    fn test_event_log_to_csv() {
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

        let mut event_log = EventLog { traces: Vec::new() };

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

        let mut event_log = EventLog { traces: Vec::new() };
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
