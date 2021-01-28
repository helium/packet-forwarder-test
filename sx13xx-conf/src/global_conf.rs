use super::File;
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, BufReader};

// Top level struct allows for the "gateway_conf" field to exist
// without getting in the way of the flexible parsing of
// SX130x_conf or SX1301_conf
#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(flatten)]
    config: Sx130xConf,
}

// This enum allows Sx1301/Sx1302 files to be parsed flexibly
#[derive(Deserialize, Serialize)]
#[allow(non_camel_case_types)]
enum Sx130xConf {
    SX130x_conf(Sx130xConfData),
    SX1301_conf(Sx130xConfData),
}

impl Config {
    pub fn summary(&self) -> String {
        match &self.config {
            Sx130xConf::SX1301_conf(sx1301) => sx1301.summary(),
            Sx130xConf::SX130x_conf(sx1302) => sx1302.summary(),
        }
    }

    pub fn frequency(&self, channel: usize) -> Option<isize> {
        match &self.config {
            Sx130xConf::SX1301_conf(sx1301) => sx1301.frequency(channel),
            Sx130xConf::SX130x_conf(sx1302) => sx1302.frequency(channel),
        }
    }

    // it is common for these JSON files to have comments in them
    // which shouldn't normally happen
    // so this helper function "cleans it up" before feeding it to serde_json
    pub fn from_file(file: File) -> Result<Config, Box<dyn std::error::Error>> {
        let reader = BufReader::new(file);

        let mut contents = String::new();
        for line_result in reader.lines() {
            // remove whitespace
            let line = line_result?.replace(' ', "");

            // remove any comments
            // this logic works for whole line of end of line
            for s in line.split("/*") {
                if !s.ends_with("*/") {
                    contents.push_str(&s);
                    contents.push('\n');
                }
            }
        }

        //println!("{}", contents);
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Sx130xConfData {
    radio_0: Radio,
    radio_1: Radio,
    chan_multiSF_0: Channel,
    chan_multiSF_1: Channel,
    chan_multiSF_2: Channel,
    chan_multiSF_3: Channel,
    chan_multiSF_4: Channel,
    chan_multiSF_5: Channel,
    chan_multiSF_6: Channel,
    chan_multiSF_7: Channel,
    chan_Lora_std: LoraStd,
    chan_FSK: ChannelFSK,
}

impl Sx130xConfData {
    fn frequency(&self, channel: usize) -> Option<isize> {
        match channel {
            0 => self.chan_multiSF_0.frequency(&self.radio_0, &self.radio_1),
            1 => self.chan_multiSF_1.frequency(&self.radio_0, &self.radio_1),
            2 => self.chan_multiSF_2.frequency(&self.radio_0, &self.radio_1),
            3 => self.chan_multiSF_3.frequency(&self.radio_0, &self.radio_1),
            4 => self.chan_multiSF_4.frequency(&self.radio_0, &self.radio_1),
            5 => self.chan_multiSF_5.frequency(&self.radio_0, &self.radio_1),
            6 => self.chan_multiSF_6.frequency(&self.radio_0, &self.radio_1),
            7 => self.chan_multiSF_7.frequency(&self.radio_0, &self.radio_1),
            8 => self.chan_Lora_std.frequency(&self.radio_0, &self.radio_1),
            9 => self.chan_FSK.frequency(&self.radio_0, &self.radio_1),
            _ => None,
        }
    }

    fn summary(&self) -> String {
        // We will confirm that all "listened to" frequencies can also be transmitted on
        // since that is a requirement for POC
        let mut frequencies = Vec::new();
        if let Some(frequency) = self.chan_multiSF_0.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_1.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_2.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_3.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_4.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_5.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_6.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }
        if let Some(frequency) = self.chan_multiSF_7.frequency(&self.radio_0, &self.radio_1) {
            frequencies.push(frequency);
        }

        // iterate through all frequencies and confirm that they are between
        // tx_freq_min and tx_freq_max
        let mut valid_tx = true;
        if let (Some(lb), Some(ub)) = (self.radio_0.tx_freq_min, self.radio_0.tx_freq_max) {
            for frequency in frequencies {
                if frequency > ub || frequency < lb {
                    valid_tx = false;
                }
            }
        } else {
            panic!("No tx freq max and min for radio_0!")
        }

        // prepare the summary to be printed
        let mut summary = String::new();
        summary.push_str("1        ");
        summary.push_str(&self.chan_multiSF_0.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n2        ");
        summary.push_str(&self.chan_multiSF_1.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n3        ");
        summary.push_str(&self.chan_multiSF_2.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n4        ");
        summary.push_str(&self.chan_multiSF_3.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n5        ");
        summary.push_str(&self.chan_multiSF_4.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n6        ");
        summary.push_str(&self.chan_multiSF_5.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n7        ");
        summary.push_str(&self.chan_multiSF_6.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\n8        ");
        summary.push_str(&self.chan_multiSF_7.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\nFat LoRa ");
        summary.push_str(&self.chan_Lora_std.summary(&self.radio_0, &self.radio_1));
        summary.push_str("\nFSK      ");
        summary.push_str(&self.chan_FSK.summary(&self.radio_0, &self.radio_1));
        if !valid_tx {
            summary.push_str("\nWARNING: Cannot transmit on all uplink frequencies for POC!");
        }
        summary
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Radio {
    freq: isize,
    tx_freq_min: Option<isize>,
    tx_freq_max: Option<isize>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Channel {
    enable: bool,
    #[serde(flatten)]
    config: Option<ChannelEnabled>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ChannelEnabled {
    r#if: isize,
    radio: usize,
}

impl Channel {
    fn frequency(&self, radio_0: &Radio, radio_1: &Radio) -> Option<isize> {
        match self.enable {
            true => {
                if let Some(config) = &self.config {
                    Some(match config.radio {
                        0 => radio_0.freq + config.r#if,
                        1 => radio_1.freq + config.r#if,
                        _ => panic!("invalid radio!"),
                    })
                } else {
                    panic!("LoRa Channel enabled but no 'radio' and/or no 'if'")
                }
            }
            false => None,
        }
    }

    fn summary(&self, radio_0: &Radio, radio_1: &Radio) -> String {
        if let Some(frequency) = self.frequency(radio_0, radio_1) {
            format!("{} MHz", frequency as f64 / 1_000_000.0)
        } else {
            "Disabled".to_string()
        }
    }
}

impl LoraStd {
    fn frequency(&self, radio_0: &Radio, radio_1: &Radio) -> Option<isize> {
        match self.enable {
            true => {
                if let Some(config) = &self.config {
                    Some(match config.radio {
                        0 => radio_0.freq + config.r#if,
                        1 => radio_1.freq + config.r#if,
                        _ => panic!("invalid radio!"),
                    })
                } else {
                    panic!("LoraStd enabled but no 'radio' and/or no 'if'")
                }
            }
            false => None,
        }
    }

    fn bandwidth(&self) -> Option<usize> {
        match self.enable {
            true => {
                if let Some(config) = &self.config {
                    Some(config.bandwidth)
                } else {
                    panic!("LoraStd enabled but no 'bandwidth'")
                }
            }
            false => None,
        }
    }

    fn summary(&self, radio_0: &Radio, radio_1: &Radio) -> String {
        if let (Some(frequency), Some(bandwidth)) =
            (self.frequency(radio_0, radio_1), self.bandwidth())
        {
            format!(
                "{} MHz, BW {} KHz",
                frequency as f64 / 1_000_000.0,
                bandwidth as f64 / 1_000.0
            )
        } else {
            "Disabled".to_string()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct LoraStd {
    enable: bool,
    #[serde(flatten)]
    config: Option<LoraStdEnabled>,
}

#[derive(Deserialize, Serialize, Debug)]
struct LoraStdEnabled {
    bandwidth: usize,
    r#if: isize,
    radio: usize,
}

#[derive(Deserialize, Serialize, Debug)]
struct ChannelFSK {
    enable: bool,
    #[serde(flatten)]
    config: Option<LoraStdEnabled>,
}

impl ChannelFSK {
    fn frequency(&self, radio_0: &Radio, radio_1: &Radio) -> Option<isize> {
        match self.enable {
            true => {
                if let Some(config) = &self.config {
                    Some(match config.radio {
                        0 => radio_0.freq + config.r#if,
                        1 => radio_1.freq + config.r#if,
                        _ => panic!("invalid radio!"),
                    })
                } else {
                    panic!("LoraStd enabled but no 'radio' and/or no 'if'")
                }
            }
            false => None,
        }
    }

    fn bandwidth(&self) -> Option<usize> {
        match self.enable {
            true => {
                if let Some(config) = &self.config {
                    Some(config.bandwidth)
                } else {
                    panic!("ChannelFSK enabled but no 'bandwidth'")
                }
            }
            false => None,
        }
    }

    fn summary(&self, radio_0: &Radio, radio_1: &Radio) -> String {
        if let (Some(frequency), Some(bandwidth)) =
            (self.frequency(radio_0, radio_1), self.bandwidth())
        {
            format!(
                "{} MHz, BW {} KHz",
                frequency as f64 / 1_000_000.0,
                bandwidth as f64 / 1_000.0
            )
        } else {
            "Disabled".to_string()
        }
    }
}