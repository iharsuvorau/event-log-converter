#![feature(slice_group_by)]

pub mod xes;
pub mod io;
pub mod conversion;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use super::*;

    fn test_log_path() -> String {
        let mut project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        project_dir.push("event_logs");
        project_dir.push("Production.xes");
        Path::new(&project_dir).to_str().unwrap().to_string()
    }

    #[test]
    fn test_parse_xes() {
        let file_path = test_log_path();
        let log = xes::lifecycle::parse_file(Path::new(&file_path), false);
        println!("{log:?}");
    }

    #[test]
    fn test_lifecycle_to_event_log() {
        let file_path = test_log_path();
        let mut log = xes::lifecycle::parse_file(Path::new(&file_path), false);
        let event_log = crate::conversion::lifecycle_to_interval(&mut log);
        // println!("{event_log:?}");
        event_log.pretty_print();
    }
}
