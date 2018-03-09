use mega_coll::conf::{app, fluentd, fs};

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-fs-report-conf",
            about = "Configuration for Rust fs-report")]
pub struct ArgConf {
    #[structopt(short = "c", long = "conf",
                default_value = "config/rs-fs-report.toml",
                help = "Configuration file path")]
    pub conf: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: app::Config,
    pub fluentd: fluentd::Config,
    pub fs: fs::Config,
}
