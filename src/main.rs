#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

#[macro_use]
extern crate failure;
extern crate fruently;
extern crate fs2;
#[macro_use]
extern crate getset;
extern crate json_collection;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_humantime;
extern crate simple_logger;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate toml;

#[cfg(test)]
#[macro_use]
extern crate indoc;

mod conf;
mod error;
mod util;

use conf::{ArgConf, Config};
use error::{ErrorKind, Result};
use failure::ResultExt;
use fs2::FsStats;
use fruently::fluent::Fluent;
use fruently::forwardable::JsonForwardable;
use fruently::retry_conf::RetryConf;
use json_collection::{Storage, StorageBuilder};
use std::path::Path;
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

fn create_and_check_fluent(conf: &Config) -> Result<Fluent<&String>> {
    let fluent_conf = RetryConf::new()
        .max(conf.fluentd.try_count)
        .multiplier(conf.fluentd.multiplier);

    let fluent_conf = match conf.fluentd.store_file_path {
        Some(ref store_file_path) => {
            fluent_conf.store_file(Path::new(store_file_path).to_owned())
        }
        None => fluent_conf,
    };

    let fluent = Fluent::new_with_conf(
        &conf.fluentd.address,
        conf.fluentd.tag.as_str(),
        fluent_conf,
    );

    fluent
        .clone()
        .post("rs-fs-report-log-initialization")
        .context(ErrorKind::FluentInitCheck)?;

    Ok(fluent)
}

fn run_impl(conf: &Config) -> Result<()> {
    let fluent = create_and_check_fluent(conf)?;
    let stats = fs2::statvfs(&conf.fs.path).context(ErrorKind::Statvfs)?;
    let storage = stats_to_storage(&conf.fs.path, &stats);

    fluent
        .clone()
        .post(&storage)
        .context(ErrorKind::FluentPostTaggedRecord)?;

    Ok(())
}

fn run(conf: &Config) -> Result<()> {
    // to check if the process is already running as another PID
    let _flock = util::lock_file(&conf.general.lock_file)?;

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

    let conf: Config = toml::from_str(&util::read_from_file(&arg_conf.conf)?)
        .context(ErrorKind::TomlConfigParse)?;

    match conf.general.log_conf_path {
        Some(ref log_conf_path) => {
            log4rs::init_file(log_conf_path, Default::default())
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
