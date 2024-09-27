use crate::config::get_config;

pub fn obfuscate_id(x: i64) -> i64 {
    let config = get_config();
    let mut mask1 = [0u8; 8];
    let mut mask2 = [0u8; 8];
    let d1 = 7;
    let d2 = 14;
    mask1.copy_from_slice(&config.id_mask()[0..64 / 8]);
    mask2.copy_from_slice(&config.id_mask()[64 / 8..]);

    let mask1 = i64::from_le_bytes(mask1);
    let mask2 = i64::from_le_bytes(mask2);

    let t = (x ^ (x >> d1)) & mask1;
    let u = x ^ t ^ (t << d1);
    let t = (u ^ (u >> d2)) & mask2;
    let y = u ^ t ^ (t << d2);

    y
}

pub fn restore_id(y: i64) -> i64 {
    let config = get_config();
    let mut mask1 = [0u8; 8];
    let mut mask2 = [0u8; 8];
    let d1 = 7;
    let d2 = 14;
    mask1.copy_from_slice(&config.id_mask()[0..64 / 8]);
    mask2.copy_from_slice(&config.id_mask()[64 / 8..]);

    let mask1 = i64::from_le_bytes(mask1);
    let mask2 = i64::from_le_bytes(mask2);

    let t = (y ^ (y >> d2)) & mask2;
    let u = y ^ t ^ (t << d2);
    let t = (u ^ (u >> d1)) & mask1;
    let z = u ^ t ^ (t << d1);

    z
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_obfuscate() {
        let id = 123458;
        let obfuscated = obfuscate_id(id);
        let real = restore_id(obfuscated);
        assert_eq!(obfuscated, real);
    }
}
