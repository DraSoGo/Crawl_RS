//! Modal screens overlaid on top of the map: inventory, death, etc.

use crossterm::style::Color;
use hecs::{Entity, World};

use crate::ecs::components::{
    AmuletEffect, Equipment, Inventory, Item, ItemKind, Name, Player, PotionEffect,
    RingEffect, ScrollKind, ThrowableKind, WandKind,
};
use crate::ui::Buffer;

pub fn draw_inventory(world: &World, buffer: &mut Buffer, cursor: usize) {
    if buffer.height() == 0 || buffer.width() == 0 {
        return;
    }
    // Translucent darken: clear to background.
    for y in 0..buffer.height() {
        for x in 0..buffer.width() {
            buffer.put(x, y, crate::ui::Cell::BLANK);
        }
    }

    let player = match world.query::<&Player>().iter().next().map(|(e, _)| e) {
        Some(p) => p,
        None => return,
    };
    let lines = build_lines(world, player, cursor, buffer.height() as usize);
    let start_y = (buffer.height() as usize)
        .saturating_sub(lines.len())
        / 2;
    for (i, (text, color)) in lines.iter().enumerate() {
        let y = (start_y + i) as u16;
        let truncated = take_chars(text, buffer.width() as usize);
        buffer.put_str(2, y, &truncated, *color, Color::Reset);
    }
}

fn build_lines(
    world: &World,
    player: Entity,
    cursor: usize,
    screen_rows: usize,
) -> Vec<(String, Color)> {
    let mut out: Vec<(String, Color)> = Vec::new();
    out.push(("Inventory".to_string(), Color::Yellow));
    out.push((String::new(), Color::Reset));
    let inv = match world.get::<&Inventory>(player) {
        Ok(i) => i.clone(),
        Err(_) => return out,
    };
    let equipment = world
        .get::<&Equipment>(player)
        .map(|e| *e)
        .unwrap_or_default();
    if inv.items.is_empty() {
        out.push(("  (empty)".to_string(), Color::DarkGrey));
    }
    let reserved_rows = 4usize;
    let visible_rows = screen_rows.saturating_sub(reserved_rows).max(1);
    let item_start = cursor.saturating_add(1).saturating_sub(visible_rows);
    let item_end = (item_start + visible_rows).min(inv.items.len());
    for (idx, entity) in inv
        .items
        .iter()
        .enumerate()
        .skip(item_start)
        .take(item_end.saturating_sub(item_start))
    {
        let name = world
            .get::<&Name>(*entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|_| "?".into());
        let descriptor = describe(world, *entity);
        let equipped = matches!(equipment.weapon, Some(w) if w == *entity)
            || matches!(equipment.armor, Some(a) if a == *entity)
            || matches!(equipment.ring, Some(r) if r == *entity)
            || matches!(equipment.amulet, Some(a) if a == *entity);
        let mark = if equipped { " [E]" } else { "" };
        let prefix = if idx == cursor { ">" } else { " " };
        let color = if idx == cursor { Color::Cyan } else { Color::White };
        out.push((format!("  {prefix} {name}{mark} {descriptor}"), color));
    }
    out.push((String::new(), Color::Reset));
    out.push((
        "  up/down select   f use/equip   g sell   esc/i close".to_string(),
        Color::DarkGrey,
    ));
    out
}

fn describe(world: &World, item: Entity) -> String {
    let kind = match world.get::<&Item>(item) {
        Ok(i) => i.kind,
        Err(_) => return String::new(),
    };
    match kind {
        ItemKind::Potion(p) => describe_potion(p),
        ItemKind::Scroll(s) => describe_scroll(s).to_string(),
        ItemKind::Weapon { attack_bonus } => format!("(+{attack_bonus} atk)"),
        ItemKind::Armor { defense_bonus } => format!("(+{defense_bonus} def)"),
        ItemKind::Ring(r) => format!("({})", describe_ring(r)),
        ItemKind::AmuletItem(a) => format!("({})", describe_amulet(a)),
        ItemKind::Wand { kind, charges } => {
            format!("({} {} charges)", describe_wand(kind), charges)
        }
        ItemKind::Throwable(t) => format!("({})", describe_throwable(t)),
        ItemKind::Food { nutrition, poisonous } => {
            if poisonous {
                format!("(+{nutrition} food, suspicious)")
            } else {
                format!("(+{nutrition} food)")
            }
        }
        ItemKind::Corpse => "(corpse)".to_string(),
    }
}

fn describe_potion(p: PotionEffect) -> String {
    match p {
        PotionEffect::Heal(n) => format!("(+{n} hp)"),
        PotionEffect::GreaterHeal(n) => format!("(+{n} hp)"),
        PotionEffect::FullHeal => "(full hp)".to_string(),
        PotionEffect::MaxHpUp(n) => format!("(+{n} max hp)"),
        PotionEffect::BuffAttack { amount, turns } => {
            format!("(+{amount} atk for {turns} turns)")
        }
        PotionEffect::BuffVision { amount, turns } => {
            format!("(+{amount} sight for {turns} turns)")
        }
        PotionEffect::CurePoison => "(cure poison)".to_string(),
    }
}

fn describe_scroll(s: ScrollKind) -> &'static str {
    match s {
        ScrollKind::Mapping => "(reveals level)",
        ScrollKind::Teleport => "(random teleport)",
        ScrollKind::Identify => "(identify)",
        ScrollKind::MagicMissile => "(zap nearest mob)",
        ScrollKind::EnchantWeapon => "(+1 weapon)",
        ScrollKind::EnchantArmor => "(+1 armor)",
        ScrollKind::Fear => "(routs nearby)",
        ScrollKind::Summon => "(summon allies)",
        ScrollKind::Light => "(extends FOV)",
        ScrollKind::Recall => "(to up-stair)",
    }
}

fn describe_ring(r: RingEffect) -> &'static str {
    match r {
        RingEffect::Regen => "regen +1/turn",
        RingEffect::Protection => "+1 def",
        RingEffect::Vision => "+2 sight",
    }
}

fn describe_amulet(a: AmuletEffect) -> &'static str {
    match a {
        AmuletEffect::TeleportControl => "controls teleports",
    }
}

fn describe_wand(k: WandKind) -> &'static str {
    match k {
        WandKind::Fire => "fire",
        WandKind::Cold => "cold",
        WandKind::Lightning => "lightning",
    }
}

fn describe_throwable(t: ThrowableKind) -> &'static str {
    match t {
        ThrowableKind::OilFlask => "ignites tile",
        ThrowableKind::SmokeBomb => "clears aggro",
    }
}

fn take_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
