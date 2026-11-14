// Auto Eat's brain: pure standard library, no game types, so it compiles and
// tests wherever — as this crate's library and, injected, as a module of the
// game. Hunger is the server's number on any world you join: nothing a
// client holds can freeze what another machine counts down. What a client
// can do is answer it — eating is a real action the server honors wherever
// it runs, so the mod picks food from the hotbar and holds use until the
// bite lands, and the drain never wins.
//
// The hold is the whole trick: a bite takes about thirty ticks, and a
// released button cancels it. The press is the game's own input mapping
// through the agent source — the same path a script's click takes — so the
// item-use is the player's own, packets and all.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from any mod.
//
// Copyright (c) 2026 hakergeniusz. MIT.

/// The mod's registry id: the Options > Mods row and the injection gates.
pub const ID: &str = "auto_eat";

/// Eat once the hunger level is at or below this: the regen band (18 and
/// up) is kept fed, and the bar never walks into the starving half.
pub const EAT_AT: f32 = 15.0;

/// Ticks the use is held before the mod gives up on the bite: the plain
/// foods' thirty-two, plus margin for the tick the press lands.
pub const EAT_TICKS: u32 = 34;

/// Foods the mod will eat, best first: cooked before raw, staples before
/// treats, golden apples last — they are too good to spend on hunger alone.
pub const FOODS: [&str; 24] = [
    "minecraft:bread",
    "minecraft:cooked_beef",
    "minecraft:cooked_porkchop",
    "minecraft:cooked_chicken",
    "minecraft:cooked_mutton",
    "minecraft:cooked_rabbit",
    "minecraft:cooked_cod",
    "minecraft:cooked_salmon",
    "minecraft:baked_potato",
    "minecraft:apple",
    "minecraft:carrot",
    "minecraft:pumpkin_pie",
    "minecraft:potato",
    "minecraft:beetroot",
    "minecraft:sweet_berries",
    "minecraft:glow_berries",
    "minecraft:porkchop",
    "minecraft:beef",
    "minecraft:mutton",
    "minecraft:rabbit",
    "minecraft:cod",
    "minecraft:salmon",
    "minecraft:golden_apple",
    "minecraft:enchanted_golden_apple",
];

/// Whether this hunger level asks for food now: at or below the threshold.
/// A level already being fed (the caller knows a bite is in flight) never
/// asks twice.
pub fn should_eat(food: f32) -> bool {
    food <= EAT_AT
}

/// Whether the item is food the mod will spend.
pub fn is_food(item: &str) -> bool {
    FOODS.contains(&item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hunger_at_the_threshold_asks_for_food() {
        assert!(should_eat(15.0));
        assert!(should_eat(0.0));
        assert!(should_eat(EAT_AT));
    }

    #[test]
    fn fed_hunger_does_not() {
        assert!(!should_eat(15.01));
        assert!(!should_eat(20.0));
    }

    #[test]
    fn the_staples_are_food() {
        for item in [
            "minecraft:bread",
            "minecraft:cooked_beef",
            "minecraft:golden_apple",
        ] {
            assert!(is_food(item), "{item}");
        }
    }

    #[test]
    fn the_traps_are_not() {
        // Raw chicken poisons, rotten flesh starves, poison is not food.
        for item in [
            "minecraft:rotten_flesh",
            "minecraft:spider_eye",
            "minecraft:poisonous_potato",
            "minecraft:pufferfish",
            "minecraft:chicken",
        ] {
            assert!(!is_food(item), "{item}");
        }
    }

    #[test]
    fn cooked_staples_rank_before_raw_and_golden() {
        let rank = |item: &str| FOODS.iter().position(|&f| f == item).unwrap();
        assert!(rank("minecraft:bread") < rank("minecraft:apple"));
        assert!(rank("minecraft:apple") < rank("minecraft:beef"));
        assert!(rank("minecraft:salmon") < rank("minecraft:golden_apple"));
        assert_eq!(rank("minecraft:enchanted_golden_apple"), FOODS.len() - 1);
    }

    #[test]
    fn the_hold_outlasts_a_bite() {
        // The plain foods' bite is thirty-two ticks; the hold must cover it
        // or the release cancels the eat mid-chew.
        let hold = EAT_TICKS;
        assert!(hold >= 33);
    }
}
