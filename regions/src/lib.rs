use strum_macros::EnumString;

#[derive(Debug, EnumString)]
#[allow(non_camel_case_types)]
pub enum Region {
    EU868,
    US915,
    CN470,
    AS923_A1,
    AS923_A2,
}

impl Region {
    pub fn get_uplink_frequencies(&self) -> &[usize] {
        match self {
            Region::EU868 => &EU868_UPLINK_FREQUENCIES,
            Region::US915 => &US915_UPLINK_FREQUENCIES,
            Region::CN470 => &CN470_UPLINK_FREQUENCIES,
            Region::AS923_A1 => &AS923_A1_UPLINK_FREQUENCIES,
            Region::AS923_A2 => &AS923_A2_UPLINK_FREQUENCIES,
        }
    }
}
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

pub const AS923_A1_UPLINK_FREQUENCIES: [usize; 9] = [
    923_200_000,
    923_400_000,
    922_200_000,
    922_400_000,
    922_600_000,
    922_800_000,
    923_000_000,
    922_000_000,
    922_100_000,
];

pub const AS923_A2_UPLINK_FREQUENCIES: [usize; 9] = [
    923_200_000,
    923_400_000,
    923_600_000,
    923_800_000,
    924_000_000,
    924_200_000,
    924_400_000,
    924_600_000,
    924_500_000,
];
