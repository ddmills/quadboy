use bevy_ecs::{hierarchy::ChildOf, world::World};

use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Palette, Rand},
    domain::{Terrain, Zone, ZoneStatus},
    rendering::{Glyph, Position, RenderLayer, TrackZone, zone_local_to_world},
    states::CleanupStatePlay,
};

pub fn gen_zone(world: &mut World, zone_idx: usize) {
    let mut rand = Rand::seed(zone_idx as u64);

    let terrains = [
        Terrain::Dirt,
        Terrain::Dirt,
        Terrain::Dirt,
        Terrain::Dirt,
        Terrain::Grass,
        Terrain::Grass,
        Terrain::Grass,
        Terrain::Grass,
        Terrain::River,
    ];

    let terrain = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_x, _y| rand.pick(&terrains));

    let zone_entity_id = world.spawn((ZoneStatus::Dormant, CleanupStatePlay)).id();

    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        let wpos = zone_local_to_world(zone_idx, x, y);
        let terrain = terrain.get(x, y).unwrap_or(&Terrain::Dirt);

        let idx = terrain.tile();
        let (bg, fg) = terrain.colors();

        // trees
        if rand.bool(0.05) && *terrain != Terrain::River {
            world.spawn((
                Position::new(wpos.0, wpos.1, wpos.2),
                Glyph::new(68, Palette::DarkGreen, Palette::Purple).layer(RenderLayer::Actors),
                ChildOf(zone_entity_id),
                ZoneStatus::Dormant,
                TrackZone,
                CleanupStatePlay,
            ));
        }

        // Add terrain tiles
        world.spawn((
            Position::new(wpos.0, wpos.1, wpos.2),
            Glyph::idx(idx)
                .bg_opt(bg)
                .fg1_opt(fg)
                .layer(RenderLayer::Ground),
            ChildOf(zone_entity_id),
            ZoneStatus::Dormant,
            CleanupStatePlay,
        ));
    });

    world
        .entity_mut(zone_entity_id)
        .insert(Zone::new(zone_idx, terrain));
}
