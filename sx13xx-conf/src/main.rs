use std::fs::File;
use std::path::Path;
use structopt::StructOpt;

use regions::Region;

mod global_conf;
use global_conf::*;

#[derive(Debug, StructOpt)]
/// Tests the frequency configuration of a SX1301
/// or SX1302 configuration file (global_conf.json)
pub struct Opt {
    /// Path to global_conf.json under test. SX1301
    /// and SX1302 configuration files are acceptable
    /// Comments (eg: "//" or "/* */ and variables
    /// (eg: ${VAR}) are stripped out before parsing
    #[structopt(name = "path_to_conf", required = true)]
    path: String,
    /// Selection region to test against. Options are:
    /// US915, EU868, EU433, CN470, CN779, AU915,
    /// AS923_1, AS923_2, AS923_3, AS923_4, KR920,
    /// IN865, RU864
    #[structopt(required = true)]
    region: Region,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opt::from_args();
    let path = Path::new(&opts.path);

    if !path.exists() {
        panic!("Path {} does not exist", &opts.path);
    }

    if path.is_file() {
        let file = File::open(&path)?;
        let config = Config::from_file(file)?;
        println!("{}", config.summary());

        let channels = opts.region.get_uplink_frequencies();

        for (index, channel) in channels.iter().enumerate() {
            if let Some(config_frequency) = config.frequency(index) {
                if &(config_frequency as usize) != channel {
                    println!(
                        "Channel {} mismatch! Expected {}, but got {}",
                        index, channel, config_frequency
                    );
                }
            } else {
                println!(
                    "Channel {} mismatch! Expected {}, but channel not configured",
                    index, channel
                );
            }
        }
    }
    Ok(())
}
