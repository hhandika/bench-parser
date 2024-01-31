use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use clap::{crate_description, crate_name, crate_version, Arg, ArgMatches, Command};
use glob::glob;

pub fn parser_arg() -> ArgMatches {
    Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author("Heru Handika")
        .arg_required_else_help(true)
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .help("Input file path")
                .multiple_values(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .default_value("result")
                .takes_value(true),
        )
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .help("Dataset size")
                .default_value("5")
                .takes_value(true),
        )
        .get_matches()
}

pub fn parse_input(matches: &ArgMatches) -> Vec<PathBuf> {
    let inputs: Vec<PathBuf> = matches
        .values_of("input")
        .expect("No input provided")
        .map(PathBuf::from)
        .collect();
    if cfg!(windows) {
        let inputs = inputs
            .iter()
            .map(|t| OsStr::new(t).to_string_lossy())
            .collect::<Vec<_>>();
        let files: Vec<PathBuf> = inputs
            .iter()
            .flat_map(|i| {
                glob(i)
                    .expect("Failed globbing files")
                    .filter_map(|ok| ok.ok())
                    .collect::<Vec<PathBuf>>()
            })
            .collect();
        assert!(!files.is_empty(), "Empty folders!");
        files
    } else {
        inputs
    }
}

pub fn parse_output(matches: &ArgMatches) -> &Path {
    Path::new(matches.value_of("output").expect("No output provided"))
}

pub fn parse_dataset_size(matches: &ArgMatches) -> usize {
    matches
        .value_of("size")
        .expect("No dataset size provided")
        .parse::<usize>()
        .expect("Failed parsing dataset size")
}
