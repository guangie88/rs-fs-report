use serde_humantime;
use std::time::Duration;
use super::krb5;

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-hdfs-report-conf",
            about = "Configuration for Rust hdfs-report")]
pub struct ArgConf {
    #[structopt(short = "c", long = "conf",
                default_value = "config/rs-hdfs-report.toml",
                help = "Configuration file path")]
    pub conf: String,
}

#[derive(Deserialize, Debug)]
pub struct Config<'a> {
    pub fluentd: FluentdConfig,
    pub general: GeneralConfig,
    pub hdfs: HdfsConfig,
    pub kinit: KInitConfig<'a>,
}

#[derive(Deserialize, Debug)]
pub struct FluentdConfig {
    pub address: String,
    pub tag: String,
    pub try_count: u64,
    pub multiplier: f64,
    pub store_file_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    pub log_conf_path: Option<String>,
    pub lock_file: String,
    #[serde(with = "serde_humantime")]
    pub repeat_delay: Option<Duration>,
}

#[derive(Deserialize, Debug)]
pub struct HdfsConfig {
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct KInitConfig<'a> {
    pub login: String,
    pub auth: krb5::Auth<'a>,
}