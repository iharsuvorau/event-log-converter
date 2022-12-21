use clap::Parser;

use event_log_converter::cli;

fn main() {
    let args = cli::Args::parse();

    match args.action {
        cli::Action::XesToCsv => {
            cli::convert_xes_to_csv(&args);
        }
        cli::Action::CsvToXes => {
            cli::convert_csv_to_xes(&args);
        }
    }
}
