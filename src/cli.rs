use clap::Parser;
use std::path::Path;
use std::io::BufWriter;
use crate::{conversion, io};
use crate::io::EventLogColumns;
use crate::xes::interval::EventLog;
use crate::xes::lifecycle;

/// Event log converter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,

    /// The input event log path
    #[arg(short, long)]
    pub input_log: String,

    /// The output directory for the converted log
    #[arg(short, long, default_value = ".")]
    pub output_dir: String,

    /// Filter out Start and End events if present in the log
    #[arg(short, long, default_value = "false")]
    pub no_start_events: bool,

    /// Case ID column name
    #[arg(short, long, default_value = "case")]
    pub case: String,

    /// Activity column name
    #[arg(short, long, default_value = "activity")]
    pub activity: String,

    /// Resource column name
    #[arg(short, long, default_value = "resource")]
    pub resource: String,

    /// Start timestamp column name
    #[arg(short, long, default_value = "start_time")]
    pub start_time: String,

    /// End timestamp column name
    #[arg(short, long, default_value = "end_time")]
    pub end_time: String,

    /// Variant column name
    #[arg(short, long, default_value = "variant")]
    pub variant: String,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    XesToCsv,
    CsvToXes,
}

pub fn convert_xes_to_csv(args: &Args) {
    let input_log = Path::new(&args.input_log);
    let output_dir = Path::new(&args.output_dir);

    let mut log = lifecycle::parse_file(input_log, args.no_start_events);
    let event_log = conversion::lifecycle_to_interval(&mut log);

    let input_log_path = Path::new(&input_log);
    let output_file_path = output_dir
        .join(input_log_path.file_name().unwrap())
        .with_extension("csv");

    let mut csv_file = std::fs::File::create(output_file_path).unwrap();
    io::interval_to_csv(&event_log, &mut csv_file);
}

pub fn convert_csv_to_xes(args: &Args) {
    let input_log = Path::new(&args.input_log);
    let output_dir = Path::new(&args.output_dir);

    let mut event_log = EventLog {
        traces: Vec::new(),
        columns: EventLogColumns {
            case: args.case.clone(),
            activity: args.activity.clone(),
            resource: args.resource.clone(),
            start_time: args.start_time.clone(),
            end_time: args.end_time.clone(),
            variant: args.variant.clone(),
        },
    };
    let bytes = std::fs::read(input_log).unwrap();
    io::csv_to_interval(&bytes, &mut event_log);

    let interval_log = conversion::interval_to_lifecycle(&event_log);

    let output_file_path = output_dir.join(input_log.file_name().unwrap()).with_extension("xes");

    let mut xes_file = BufWriter::new(std::fs::File::create(output_file_path).unwrap());
    io::lifecycle_to_xes(&interval_log, &mut xes_file);
}
