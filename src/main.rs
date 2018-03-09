#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

extern crate failure;
extern crate fruently;
extern crate fs2;
extern crate json_collection;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate mega_coll;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate simple_logger;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

mod conf;

use conf::{ArgConf, Config};
use mega_coll::error::{ErrorKind, Result};
use mega_coll::error::custom::PathError;
use mega_coll::util::app::{create_and_check_fluent, read_config_file};
use mega_coll::util::fs::lock_file;
use failure::ResultExt;
use fs2::FsStats;
use fruently::forwardable::JsonForwardable;
use json_collection::{Storage, StorageBuilder};
use std::process;
use std::thread;
use structopt::StructOpt;

fn stats_to_storage<P>(path: P, stats: &FsStats) -> Storage
where
    P: AsRef<str>,
{
    StorageBuilder::default()
        .path(path.as_ref())
        .capacity(stats.total_space())
        .used(stats.total_space() - stats.free_space())
        .build()
}

fn run_impl(conf: &Config) -> Result<()> {
    let fluent = create_and_check_fluent(
        &conf.fluentd,
        "rs-fs-report-log-initialization",
    )?;

    let stats = fs2::statvfs(&conf.fs.path)
        .map_err(|e| PathError::new(&conf.fs.path, e))
        .context(ErrorKind::Statvfs)?;

    let storage = stats_to_storage(&conf.fs.path, &stats);
    debug!("```\n{:#?}```", storage);

    fluent
        .clone()
        .post(&storage)
        .context(ErrorKind::FluentPostTaggedRecord)?;

    Ok(())
}

fn run(conf: &Config) -> Result<()> {
    // to check if the process is already running as another PID
    let _flock = lock_file(&conf.general.lock_file)?;

    match conf.general.repeat_delay {
        Some(repeat_delay) => loop {
            print_run_status(&run_impl(conf));
            thread::sleep(repeat_delay)
        },
        None => run_impl(conf),
    }
}

fn init() -> Result<Config> {
    let arg_conf = ArgConf::from_args();
    let conf: Config = read_config_file(&arg_conf.conf)?;

    match conf.general.log_conf_path {
        Some(ref log_conf_path) => {
            log4rs::init_file(log_conf_path, Default::default())
                .map_err(|e| PathError::new(log_conf_path, e))
                .context(ErrorKind::SpecializedLoggerInit)?
        }
        None => simple_logger::init().context(ErrorKind::DefaultLoggerInit)?,
    }

    Ok(conf)
}

fn print_run_status(res: &Result<()>) {
    match *res {
        Ok(_) => info!("Session completed!"),
        Err(ref e) => {
            error!("{}", e);
        }
    }
}

fn main() {
    let conf_res = init();

    if let Err(ref e) = conf_res {
        eprintln!("{}", e);
    }

    let res = conf_res.and_then(|conf| {
        info!("Program started!");
        debug!("```\n{:#?}```", conf);
        run(&conf)
    });

    print_run_status(&res);

    if res.is_err() {
        process::exit(1);
    }
}
