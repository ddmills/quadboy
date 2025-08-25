The overworld works on a higher level than individual tiles, it operates mostly on zones. It contains information about:

1. The biome of each zone
2. Which zones contains rivers and footpaths and how they connect
3. Where towns and other points of interest are located
4. Where stairs are located

Some of this might need to be generated at a high level when the Overworld is created, like deciding which
zones will contain towns, and which zones rivers/footpaths will pass through.

This information can be retrieved from the Overworld by calling get_overworld_zone. The OverworldZone
object is used in the granular zone generation, in the form of constraints. Each side of a zone,
north, south, east, west, up, and down, can all have some constraints dictated by the overworld, that
the individual zone builder must adhere to. These constraints must not be violated.

