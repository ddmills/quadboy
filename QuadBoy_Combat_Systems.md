# QuadBoy Combat & Character Systems

QuadBoy uses a comprehensive attribute-based character system that drives all combat calculations, weapon proficiency, and character progression. This document provides a high-level overview of how these interconnected systems work together.

## Attribute System

The foundation of character progression is built on four core attributes that define a character's fundamental capabilities:

- **Strength**: Raw physical power affecting melee weapons and health
- **Dexterity**: Agility and precision affecting ranged weapons and evasion
- **Constitution**: Physical resilience affecting health and survivability
- **Intelligence**: Mental acuity affecting armor regeneration and magical abilities

### Attribute Points
Players gain attribute points through leveling (5 + current level points available). These points can be allocated freely among the four attributes, with the ability to reallocate during character development. The system encourages specialization while allowing for hybrid builds.

## Stats System

Attributes are transformed into actionable statistics through a comprehensive calculation system:

### Core Combat Stats
- **Fortitude**: Determines maximum health points (Constitution-based)
- **Speed**: Affects movement energy costs and turn order (Dexterity-based)
- **Armor**: Maximum armor points for damage absorption (equipment-based)
- **Armor Regen**: Rate of armor point regeneration over time (Intelligence-based)
- **Dodge**: Evasion ability against incoming attacks (Dexterity-based)

### Weapon Proficiency Stats
Each weapon family has a corresponding proficiency stat that affects accuracy and effectiveness:
- **Rifle Proficiency**: Long-range precision weapons (Dexterity-based)
- **Shotgun Proficiency**: Close-range spread weapons (Strength-based)
- **Pistol Proficiency**: Balanced sidearm weapons (Strength-based)
- **Blade Proficiency**: Sharp melee weapons (Dexterity-based)
- **Cudgel Proficiency**: Blunt melee weapons (Strength-based)
- **Unarmed Proficiency**: Hand-to-hand combat (Strength-based)

### Stat Calculation
Final stat values are calculated as: `Base Attribute Value + Equipment Modifiers + Intrinsic Modifiers`. This allows for both character progression through attributes and temporary enhancement through equipment.

## Weapon Families & Types

The weapon system is organized into two primary classifications:

### Weapon Families
Six distinct weapon families each emphasize different combat approaches and attribute synergies. Each family has specialized mechanics and optimal use cases, encouraging tactical weapon selection based on situation and character build.

### Weapon Types
- **Melee Weapons**: Close-range combat with no ammunition requirements
- **Ranged Weapons**: Distance combat with ammunition management and reload mechanics

Ranged weapons feature additional complexity including range limitations, clip sizes, reload costs, and ammunition tracking, while melee weapons offer consistent availability with positioning requirements.

## Combat Formulas

### Hit/Miss Resolution
Combat accuracy uses opposed roll mechanics:
1. **Attacker Roll**: d12 + Weapon Proficiency Stat
2. **Defender Roll**: d12 + Dodge Stat
3. **Critical Hits**: Natural 12 on attacker roll always hits regardless of defense
4. **Success**: Attacker roll ≥ Defender roll (or critical hit)

### Damage System
- **Damage Calculation**: Uses dice notation (e.g., "1d6+2", "2d6+1") rolled on successful hits
- **Armor Mechanics**: Armor absorbs damage first before affecting health points
- **Health Formula**: Maximum HP = (Level × 2) + (Fortitude × 2) + 5
- **Material Damage**: Weapons specify which material types they can damage (Flesh, Wood, Stone, etc.)

### Energy & Action Costs
The turn-based system uses energy costs for all actions:
- **Movement**: Variable based on Speed stat
- **Melee Attack**: Standard energy cost
- **Ranged Attack**: Higher energy cost
- **Reload**: Weapon-specific energy cost

### Experience & Progression
Experience gain uses a level-differential formula:
```
XP Gain = (max(0, (9 + enemy_level - player_level)) / 9)³ × 120
```
This creates diminishing returns for fighting lower-level enemies while maintaining rewards for challenging encounters.

### Health & Armor Mechanics
- **Damage Priority**: Armor absorbs damage first, overflow damages health
- **Armor Regeneration**: Passive regeneration based on Intelligence after avoiding damage
- **Death Condition**: Health reaching 0 or below
- **Healing**: Restoration limited to maximum health based on level and Fortitude

## System Integration

These systems work together to create tactical depth:
- **Build Diversity**: Different attribute focuses enable varied playstyles
- **Equipment Synergy**: Gear modifiers complement and enhance attribute choices
- **Combat Tactics**: Weapon selection, positioning, and timing all matter
- **Risk/Reward**: Higher-level encounters provide greater experience but require better preparation

The interconnected nature of these systems ensures that character development, equipment choices, and combat tactics all influence each other, creating meaningful decisions at every level of play.