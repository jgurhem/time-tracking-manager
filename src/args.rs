use std::{env, ffi::OsString, path::PathBuf};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::utils::end_of_month;

pub fn parse_args() -> Args {
    let args = with_default_argfile(env::args_os().collect(), default_argfile());
    let args = argfile::expand_args_from(args.into_iter(), argfile::parse_fromfile, '@')
        .expect("failed to expand args");
    Args::parse_from(args)
}

/// `$XDG_CONFIG_HOME/ttm/args`, or `~/.config/ttm/args` when `XDG_CONFIG_HOME` is not set
fn default_argfile() -> Option<PathBuf> {
    let config_dir = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config_dir.join("ttm").join("args"))
}

/// Inserts the default argfile before command line arguments so that they take precedence
fn with_default_argfile(mut args: Vec<OsString>, argfile: Option<PathBuf>) -> Vec<OsString> {
    if let Some(argfile) = argfile.filter(|path| path.is_file()) {
        eprintln!("Reading default arguments from {}", argfile.display());
        let mut arg = OsString::from("@");
        arg.push(argfile);
        args.insert(1.min(args.len()), arg);
    }
    args
}

fn start_month() -> DateTime<Utc> {
    let utc = Utc::now();
    Utc.with_ymd_and_hms(utc.year(), utc.month(), 1, 0, 0, 0)
        .unwrap()
}

fn end_month() -> DateTime<Utc> {
    end_of_month(&start_month())
}

#[derive(Parser, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[command(version, about, long_about = None, args_override_self = true)]
pub struct Args {
    /// Provider used to retrieve entries
    #[arg(short('P'), long)]
    pub provider: String,

    /// Options passed to the provider such as authentication informations and token
    /// ---
    /// *Clockify Options*
    ///   token: Clockify authentication token
    #[arg(short, long)]
    #[serde(default)]
    pub provider_options: Vec<String>,

    /// DateTime from wich to start retrieving entries
    #[arg(short, long, default_value_t = start_month())]
    #[serde(default = "start_month")]
    pub start: DateTime<Utc>,

    /// DateTime until entries are retrieved
    #[arg(short, long, default_value_t = end_month())]
    #[serde(default = "end_month")]
    pub end: DateTime<Utc>,

    /// Include entries with "Ignore" tag
    #[arg(short, long, default_value_t = false)]
    #[serde(default)]
    pub ignored: bool,

    /// Include non billable entries
    #[arg(short, long, default_value_t = false)]
    #[serde(default)]
    pub billable: bool,

    /// Projects and tasks to ignore during computations. 'Project' ignores all tasks from the project. 'Project___' ignores empty tasks. 'Project___Task' ignore the given task.
    #[arg(short('I'), long, default_values_t = Vec::<String>::new())]
    #[serde(default)]
    pub ignore_list: Vec<String>,

    /// 'Project1___Task1=Project2___Task2' allows to rename Project1 Task1 into Project2 Task2 before Tabler step
    #[arg(short, long, default_values_t = Vec::<String>::new())]
    #[serde(default)]
    pub rename: Vec<String>,

    /// 'Project1___Task1=Display' allows to rename Project1 Task1 into Display during export step
    #[arg(short, long, default_values_t = Vec::<String>::new())]
    #[serde(default)]
    pub display: Vec<String>,

    /// Number of slots per day
    #[arg(short, long, default_value_t = default_granularity())]
    #[serde(default = "default_granularity")]
    pub granularity: u8,

    /// Number of slots per day
    #[arg(long, default_value = "")]
    #[serde(default)]
    pub period: String,

    /// Options passed to exporters (e.g. aggregation_level=project)
    #[arg(short('E'), long)]
    #[serde(default)]
    pub exporter_options: Vec<String>,
}

const fn default_granularity() -> u8 {
    100
}

impl Default for Args {
    fn default() -> Self {
        Self {
            provider: "clockify".into(),
            provider_options: Default::default(),
            start: start_month(),
            end: end_month(),
            ignored: false,
            billable: false,
            ignore_list: Default::default(),
            rename: Default::default(),
            display: Default::default(),
            granularity: default_granularity(),
            period: String::from(""),
            exporter_options: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn default_deserialization() {
        let args = Args::default();
        assert_eq!(
            args,
            serde_json::from_str("{\"provider\":\"clockify\"}")
                .expect("valid json representing Args")
        )
    }

    #[test]
    fn command_line_overrides_default_argfile() {
        let argfile = env::temp_dir().join("ttm_command_line_overrides_default_argfile");
        std::fs::write(&argfile, "-Pclockify\n-ptoken=secret\n-g4\n-IProject").unwrap();

        let args = with_default_argfile(os_args(&["ttm", "-g8", "-IOther"]), Some(argfile.clone()));
        let args =
            argfile::expand_args_from(args.into_iter(), argfile::parse_fromfile, '@').unwrap();
        let args = Args::parse_from(args);
        std::fs::remove_file(&argfile).unwrap();

        assert_eq!(args.provider, "clockify");
        assert_eq!(args.provider_options, vec!["token=secret"]);
        assert_eq!(args.granularity, 8);
        assert_eq!(args.ignore_list, vec!["Project", "Other"]);
    }

    #[test]
    fn missing_default_argfile_is_ignored() {
        let args = os_args(&["ttm", "-Pclockify"]);
        let missing = env::temp_dir().join("ttm_missing_default_argfile");

        assert_eq!(with_default_argfile(args.clone(), Some(missing)), args);
        assert_eq!(with_default_argfile(args.clone(), None), args);
    }
}
