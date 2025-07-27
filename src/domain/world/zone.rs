use bevy_ecs::prelude::*;

use crate::{cfg::{MAP_SIZE, ZONE_SIZE}, common::{Grid, Palette}, domain::Zone, rendering::{zone_local_to_world, Glyph, Position, RenderLayer}};


#[derive(Event)]
pub struct LoadZoneEvent(pub usize);

#[derive(Event)]
pub struct UnloadZoneEvent(pub usize);

pub fn on_load_zone(mut cmds: Commands, mut e_load_zone: EventReader<LoadZoneEvent>)
{
    for e in e_load_zone.read() {
        let zone_idx = e.0;

        let zone_e = cmds.spawn(()).id();

        let tiles = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            let wpos = zone_local_to_world(zone_idx, x, y);

            cmds.spawn((
                Position::new(wpos.0, wpos.1),
                Glyph::new(x + y, Palette::Brown, Palette::Green)
                    .layer(RenderLayer::Ground),
            )).id()
        });

        cmds.entity(zone_e).insert(Zone::new(zone_idx, tiles));
    }
}

pub fn on_unload_zone()
{

}
