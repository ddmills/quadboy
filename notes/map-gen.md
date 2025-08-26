-- OVERWORLD STEPS (when a new game starts)
1. determine zone biomes
2. determine river locations
3. determine Point of Interest locations
4. determine road locations

-- GRANULAR STEPS (when a zone is loaded)
1. determine edge continuity
    - can be based on biome type (mountains = more rock edges)
    - get footpath entrances/exits
    - get stairs
    - get river entrances/exits
2. apply stair locations
    - spawn up/down stairs
3. apply rivers
    - rivers constraints should all connect
    - connect major rivers first
        - smaller rivers then connect into major rivers
    - rivers should avoid stairs
3. apply roads
    - all roads should connect
    - connect major roads first
        - smaller roads should connect with bigger roads
        - lastly, connect stairs to nearest road
4. apply biome builder layer, this places all natural biome stuff (terrain, rocks, trees, cactii, etc)
    - this must respect the zone constraints (i.e, Rock edges _MUST_ contain rocks. None tile MUST contain basic terrain.)
    - this must respect footpaths, rivers, and stairs
    - the biome provides information about how things are styled (i,e the terrain used for roads, the type of rock used for walls, what type of foliage and enemies might be present.)

-- TERMS
1. Zone Continuity: Rules that zone generation must follow along it's North/South/East/West edges, and any stairs that go Up/Down. This includes rivers, footpaths, and rocks.
2. Z-level: As z-level decreases, 