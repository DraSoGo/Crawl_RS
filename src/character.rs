pub const BASE_INVENTORY_SLOTS: usize = 26;
pub const INVENTORY_SLOTS_PER_LEVEL: usize = 5;

pub fn inventory_capacity(level: u32) -> usize {
    BASE_INVENTORY_SLOTS
        + level.saturating_sub(1) as usize * INVENTORY_SLOTS_PER_LEVEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_capacity_scales_with_level() {
        assert_eq!(inventory_capacity(1), 26);
        assert_eq!(inventory_capacity(2), 31);
        assert_eq!(inventory_capacity(5), 46);
    }
}
