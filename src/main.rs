use std::fmt::{Display, Formatter};
use anyhow::Result;
use clap::Parser;
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit_number: u8,
}

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about= None)]
struct Opts {
    #[command(subcommand)]
    cmd: SubCommand,
}
#[derive(Debug, Parser)]
enum SubCommand {
    #[command(name = "csv")]
    Csv(CsvOpts),
}

#[derive(Debug, Parser)]
struct CsvOpts {
    #[arg(short, long, value_parser = verify_input_file)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long, value_parser = parse_format, default_value = "json")]
    format: OutputFormat,

    #[arg(short, long, default_value_t = ',')]
    delimiter: char,

    #[arg(long, default_value_t = true)]
    header: bool,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Json,
    Yaml,
    Toml,
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)
        },
    }
}

fn verify_input_file(file_name: &str) -> Result<String, String> {
    if std::path::Path::new(file_name).exists() {
        Ok(file_name.into())
    } else {
        Err(format!("File not found: {}", file_name))
    }
}

fn process_csv(input: &str, output: String, format: OutputFormat) -> Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_value);
    }
    let json = serde_json::to_string_pretty(&ret)?;
    fs::write(output, json)?;
    Ok(())
}

fn parse_format(input: &str) -> Result<OutputFormat, anyhow::Error> {
    input.parse()
}

impl From<OutputFormat> for &'static str {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Json => "json",
            OutputFormat::Toml => "toml",
            OutputFormat::Yaml => "yaml",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "toml" => Ok(OutputFormat::Toml),
            _ => anyhow::bail!("Unrecognized format: {}", s),
        }
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}