use serde::{Deserialize, Serialize};

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
                println!("    Start: {}", event.start_time);
                println!("    End: {}", event.end_time);
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
    pub start_time: String,
    pub end_time: String,
}
