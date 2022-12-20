use clap::Parser;
use event_log_converter::io;

/// Event log converter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
    /// The input event log path
    #[arg(short, long)]
    input_log: String,
    /// The output directory for the converted log
    #[arg(short, long, default_value = ".")]
    output_dir: String,
    /// Filter out Start and End events if present in the log
    #[arg(short, long, default_value = "false")]
    no_start_events: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    XesToCsv,
}

fn main() {
    let args = Args::parse();
    let input_log = args.input_log;
    let output_dir = args.output_dir;
    let filter_start_end_events = args.no_start_events;

    match args.action {
        Action::XesToCsv => {
            io::convert_xes_to_csv(&input_log, &output_dir, filter_start_end_events);
        }
    }
}
