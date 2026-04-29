pub fn inventory_capacity(level: u32) -> usize {
    crate::config::PROGRESSION.inventory_base_slots
        + level.saturating_sub(1) as usize * crate::config::PROGRESSION.inventory_slots_per_level
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
