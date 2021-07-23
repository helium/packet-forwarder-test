use strum_macros::EnumString;

#[derive(Debug, EnumString)]
#[allow(clippy::upper_case_acronyms)]
pub enum Region {
    US915,
    EU868,
    EU433,
    CN470,
    AS923_1,
    AS923_2,
    AS923_3,
    IN865,
}

impl Region {
    pub fn get_uplink_frequencies(&self) -> &[usize] {
        match self {
            Region::US915 => &US915_UPLINK_FREQUENCIES,
            Region::EU868 => &EU868_UPLINK_FREQUENCIES,
            Region::EU433 => &EU433_UPLINK_FREQUENCIES,
            Region::CN470 => &CN470_UPLINK_FREQUENCIES,
            Region::AS923_1 => &AS923_1_UPLINK_FREQUENCIES,
            Region::AS923_2 => &AS923_2_UPLINK_FREQUENCIES,
            Region::AS923_3 => &AS923_3_UPLINK_FREQUENCIES,
            Region::IN865 => &IN865_UPLINK_FREQUENCIES,

        }
    }
}

pub const US915_UPLINK_FREQUENCIES: [usize; 8] = [
    903_900_000,
    904_100_000,
    904_300_000,
    904_500_000,
    904_700_000,
    904_900_000,
    905_100_000,
    905_300_000,
];

pub const EU868_UPLINK_FREQUENCIES: [usize; 9] = [
    868_100_000,
    868_300_000,
    868_500_000,
    867_100_000,
    867_300_000,
    867_500_000,
    867_700_000,
    867_900_000,
    868_300_000,
];

pub const EU433_UPLINK_FREQUENCIES: [usize; 3] = [
    433_175_000,
    433_375_000,
    433_575_000
];

pub const CN470_UPLINK_FREQUENCIES: [usize; 8] = [
    486_300_000,
    486_500_000,
    486_700_000,
    486_900_000,
    487_100_000,
    487_300_000,
    487_500_000,
    487_700_000,
];

pub const AS923_1_UPLINK_FREQUENCIES: [usize; 8] = [
    923_200_000,
    923_400_000,
    923_600_000,
    923_800_000,
    924_000_000,
    924_200_000,
    924_400_000,
    924_600_000,
];

pub const AS923_2_UPLINK_FREQUENCIES: [usize; 8] = [
    921_400_000,
    921_600_000,
    921_800_000,
    922_000_000,
    922_200_000,
    922_400_000,
    922_600_000,
    922_800_000,
];

pub const AS923_3_UPLINK_FREQUENCIES: [usize; 8] = [
    916_600_000,
    916_800_000,
    917_000_000,
    917_200_000,
    917_400_000,
    917_600_000,
    917_800_000,
    918_000_000,
];

pub const IN865_UPLINK_FREQUENCIES: [usize; 3] = [
    865_062_500,
    865_402_500,
    865_985_000
];

