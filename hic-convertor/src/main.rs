use std::error::Error;
use std::io;
use std::path::Path;

use log::info;
use fern;
use clap::{Arg, App, SubCommand};
use hic_convertor::{full_pipeline, convert_bam_to_pairs, deduplicate_pairs, sort_pairs};

fn setup_logging(verbosity: u64, log_file: &Path) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => base_config.level(log::LevelFilter::Info),
        1 => base_config.level(log::LevelFilter::Debug),
        _ => base_config.level(log::LevelFilter::Trace),
    };

    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(log_file)?);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%H:%M"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("convertor")
        .version("0.1.0")
        .author("Pavel Avdeyev")
        .about("hic-convertor converts BAM files with Hi-C reads to Hi-C pairs. \
                It is also sorts and deduplicates obtained Hi-C reads.")
        .subcommand(
            SubCommand::with_name("all")
                .about("Convert, sort and deduplicate Hi-C pairs.")
                .arg(
                    Arg::with_name("bam")
                        .short("b")
                        .long("bam")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("Alignments in bam format.")
                )
                .arg(
                    Arg::with_name("out")
                        .short("o")
                        .long("out_dir")
                        .value_name("DIR")
                        .takes_value(true)
                        .required(true)
                        .help("Path to output directory.")
                )
                .arg(
                    Arg::with_name("graph")
                        .short("g")
                        .long("graph")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(false)
                        .help("Path to graph in gfa format.")
                )
                .arg(
                    Arg::with_name("nproc")
                        .short("t")
                        .long("nproc")
                        .value_name("NUM")
                        .takes_value(true)
                        .required(false)
                        .help("Number of processes for sorting.")
                )
        )
        .subcommand(
            SubCommand::with_name("convert")
                .about("Convert bam to pairs file.")
                .arg(
                    Arg::with_name("bam")
                        .short("b")
                        .long("bam")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("Alignments in bam format.")
                )
                .arg(
                    Arg::with_name("pairs")
                        .short("p")
                        .long("pairs")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("File where obtained pairs will be saved.")
                )
                .arg(
                    Arg::with_name("graph")
                        .short("g")
                        .long("graph")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(false)
                        .help("Path to graph in gfa format.")
                )
        )
        .subcommand(
            SubCommand::with_name("sort")
                .about("Sort pairs file using sort command (see man sort).")
                .arg(
                    Arg::with_name("in_pairs")
                        .short("p")
                        .long("in_pairs")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("Input file with pairs.")
                )
                .arg(
                    Arg::with_name("out_pairs")
                        .short("o")
                        .long("out_pairs")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("Output file with sorted pairs.")
                )
                .arg(
                    Arg::with_name("nproc")
                        .short("t")
                        .long("nproc")
                        .value_name("NUM")
                        .takes_value(true)
                        .required(false)
                        .help("Number of processes for sorting.")
                )
        )
        .subcommand(
            SubCommand::with_name("dedup")
                .about("Remove duplicated Hi-C reads from file.")
                .arg(
                    Arg::with_name("in_pairs")
                        .short("p")
                        .long("in_pairs")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("Input file with pairs.")
                )
                .arg(
                    Arg::with_name("out_pairs")
                        .short("o")
                        .long("out_pairs")
                        .value_name("FILE")
                        .takes_value(true)
                        .required(true)
                        .help("Output file with sorted pairs.")
                )
        )
        .get_matches();

    match matches.subcommand() {
        ("all", Some(all_matches)) => {
            setup_logging(1, "convert.log".as_ref()).expect("failed to initialize logging.");
            let bam_file = all_matches.value_of("bam").unwrap();
            let out_dir = all_matches.value_of("out").unwrap();
            let nproc: u8 = all_matches.value_of("nproc").unwrap_or("4").parse().unwrap();
            info!("all with {} {} {}", bam_file, out_dir, nproc);
            match all_matches.value_of("graph") {
                None =>  full_pipeline(Path::new(bam_file), None, Path::new(out_dir), nproc)?,
                Some(_) => full_pipeline(Path::new(bam_file), None, Path::new(out_dir), nproc)?,
            }
        },
        ("convert", Some(convert_matches)) => {
            setup_logging(1, "convert.log".as_ref()).expect("failed to initialize logging.");
            let bam_file = convert_matches.value_of("bam").unwrap();
            let pairs_file = convert_matches.value_of("pairs").unwrap();
            info!("convert with {} {}", bam_file, pairs_file);
            match convert_matches.value_of("graph") {
                None =>  convert_bam_to_pairs(Path::new(bam_file), None, Path::new(pairs_file), Path::new("stats.txt"))?,
                Some(_) => convert_bam_to_pairs(Path::new(bam_file), None, Path::new(pairs_file), Path::new("stats.txt"))?,
            }
        },
        ("sort", Some(sort_matches)) => {
            setup_logging(1, "convert.log".as_ref()).expect("failed to initialize logging.");
            let in_file = sort_matches.value_of("in_pairs").unwrap();
            let out_file = sort_matches.value_of("out_pairs").unwrap();
            let nproc: u8 = sort_matches.value_of("nproc").unwrap_or("4").parse().unwrap();
            info!("sort with {} {} {}", in_file, out_file, nproc);
            sort_pairs(Path::new(in_file), Path::new(out_file), Option::from(Path::new("tmp_sort_dir")), nproc)?;
        },
        ("dedup", Some(dedup_matches)) => {
            setup_logging(1, "convert.log".as_ref()).expect("failed to initialize logging.");
            let in_file = dedup_matches.value_of("in_pairs").unwrap();
            let out_file = dedup_matches.value_of("out_pairs").unwrap();
            info!("sort with {} {}", in_file, out_file);
            deduplicate_pairs(Path::new(in_file), Path::new(out_file));
        }
        ("", None) => println!("None subcommand was used. See help for available one."),
        _ => unreachable!(),
    };
    Ok(())
}

// setup_logging(3, "all_log.log".as_ref()).expect("failed to initialize logging.");
// hicproj::run("tig_sizes.tsv")
//hicproj::run("comp18_lens.tsv")
// hicproj::run_graph("comp18.gfa")
// info!("MyProgram v0.0.1 starting up!");
// hicproj::run_matrix("comp18_lens.tsv", "pairs18.txt")
//hicproj::run_scaffolding("comp18.gfa", "rs_test3.cool")