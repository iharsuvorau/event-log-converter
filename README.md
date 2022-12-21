# Event Log Converter

[![crates.io](https://img.shields.io/crates/v/event_log_converter.svg)](https://crates.io/crates/event_log_converter)

CLI tool to convert event logs from one format to another fast and efficiently.

Supports the following conversions:

* XES to CSV
* CSV to XES

## Usage

```bash
$ event-log-converter -i filename.xes xes-to-csv
```

More on usage:

```
Usage: event_log_converter [OPTIONS] --input-log <INPUT_LOG> <COMMAND>

Commands:
  xes-to-csv  
  csv-to-xes  
  help        Print this message or the help of the given subcommand(s)

Options:
  -i, --input-log <INPUT_LOG>    The input event log path
  -o, --output-dir <OUTPUT_DIR>  The output directory for the converted log [default: .]
  -n, --no-start-events          Filter out Start and End events if present in the log
  -c, --case <CASE>              Case ID column name [default: case]
  -a, --activity <ACTIVITY>      Activity column name [default: activity]
  -r, --resource <RESOURCE>      Resource column name [default: resource]
  -s, --start-time <START_TIME>  Start timestamp column name [default: start_time]
  -e, --end-time <END_TIME>      End timestamp column name [default: end_time]
  -v, --variant <VARIANT>        Variant column name [default: variant]
  -h, --help                     Print help information
  -V, --version                  Print version information

```