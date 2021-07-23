use super::File;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

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
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
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

    pub fn from_file(mut file: File) -> Result<Config, Box<dyn std::error::Error>> {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let decommented_content = decomment(&contents);
        let config = serde_json::from_str(&decommented_content)?;
        Ok(config)
    }
}

/// Removes both c-style block comments and c++-style line comments from a str.
pub fn decomment(src: &str) -> String {
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut variable = false;
    let mut decommented = String::with_capacity(src.len());
    let mut itr = src.chars().peekable();
    while let Some(ch) = itr.next() {
        match (ch, itr.peek()) {
            ('$', Some('{')) => {
                assert!(!in_line_comment);
                assert!(!in_block_comment);
                let _ = itr.next();
                variable = true;
            }
            (_, Some('}')) => {
                if variable {
                    assert!(!in_line_comment);
                    assert!(!in_block_comment);
                    decommented.push_str("\"\"");
                    let _ = itr.next();
                    variable = false;
                } else if !in_block_comment && !in_line_comment && !ch.is_whitespace() {
                    decommented.push(ch)
                }
            }
            ('/', Some('*')) => {
                assert!(!in_line_comment);
                assert!(!in_block_comment);
                let _ = itr.next();
                in_block_comment = true;
            }
            ('*', Some('/')) => {
                assert!(in_block_comment);
                let _ = itr.next();
                in_block_comment = false;
            }
            ('/', Some('/')) => {
                assert!(!in_line_comment);
                assert!(!in_block_comment);
                let _ = itr.next();
                in_line_comment = true;
            }
            ('\r', Some('\n')) if in_line_comment => {
                let _ = itr.next();
                in_line_comment = false;
            }
            ('\n', _) if in_line_comment => {
                in_line_comment = false;
            }
            _ if in_block_comment || in_line_comment || variable => (),
            _ => decommented.push(ch),
        }
    }
    assert!(!in_block_comment);
    decommented
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
    chan_FSK: ChannelFsk,
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
        if !self.enable {
            return None;
        }
        let &ChannelEnabled { r#if, radio } = self
            .config
            .as_ref()
            .expect("LoRa Channel enabled but no 'radio' and/or no 'if'");
        Some(match radio {
            0 => radio_0.freq + r#if,
            1 => radio_1.freq + r#if,
            _ => panic!("invalid radio!"),
        })
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
struct ChannelFsk {
    enable: bool,
    #[serde(flatten)]
    config: Option<LoraStdEnabled>,
}

impl ChannelFsk {
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
