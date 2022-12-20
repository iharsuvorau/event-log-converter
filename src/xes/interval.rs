use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct EventLog<'a> {
    pub traces: Vec<Trace<'a>>,
}

impl EventLog<'_> {
    pub fn pretty_print(&self) {
        for trace in &self.traces {
            println!("Trace: {}", trace.case);
            println!("Variant: {}", trace.variant);
            for event in &trace.events {
                println!("  Event: {}", event.activity);
                println!("    Resource: {}", event.resource);
                println!("    Start: {}", event.start_time);
                println!("    End: {}", event.end_time);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace<'a> {
    pub case: &'a str,
    pub variant: &'a str,
    pub events: Vec<Event<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<'a> {
    pub activity: &'a str,
    pub resource: &'a str,
    pub start_time: &'a str,
    pub end_time: &'a str,
}
