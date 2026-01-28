mod events;

use crate::prelude::*;

pub use events::*;

pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(events::match_to_summon)
            .add_observer(events::summon_unit)
            .add_observer(events::handle_skill_orb)
            .add_observer(events::handle_mana_supply);
    }
}
