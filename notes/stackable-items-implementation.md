# Stackable Items Implementation Plan

## Overview
Implement a stackable item system that allows items like gold nuggets to stack in the inventory while maintaining the existing entity-based architecture.

## Core Design Principles
- Keep the existing inventory system with stable IDs
- Use entities to represent stacks (entity-based approach)
- Leverage existing components (Glyph, Label, Item) for stack entities
- Maintain save/load compatibility

## Components

### StackableType Enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StackableType {
    GoldNugget,
    // Future: Arrow, Coin, IronOre, etc.
}
```

### Stackable Component
```rust
#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Stackable {
    pub stack_type: StackableType,
}

impl Stackable {
    pub fn new(stack_type: StackableType) -> Self {
        Self { stack_type }
    }
}
```

### StackCount Component
```rust
#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct StackCount {
    pub count: u32,
}

impl StackCount {
    pub const MAX_STACK_SIZE: u32 = 99;
    
    pub fn new(count: u32) -> Self {
        Self { 
            count: count.min(Self::MAX_STACK_SIZE) 
        }
    }
    
    pub fn add(&mut self, amount: u32) -> u32 {
        let space = Self::MAX_STACK_SIZE - self.count;
        let to_add = amount.min(space);
        self.count += to_add;
        amount - to_add  // Return overflow
    }
    
    pub fn remove(&mut self, amount: u32) -> u32 {
        let to_remove = amount.min(self.count);
        self.count -= to_remove;
        to_remove  // Return actual amount removed
    }
    
    pub fn is_full(&self) -> bool {
        self.count >= Self::MAX_STACK_SIZE
    }
}
```

## Implementation Steps

### 1. Create Components
- Add `StackableType` enum in `src/domain/components/mod.rs`
- Add `Stackable` and `StackCount` components
- Export from components module

### 2. Update PrefabBuilder
Add new method to `PrefabBuilder`:
```rust
pub fn with_stackable(self, stack_type: StackableType, count: u32) -> Self {
    self.world.entity_mut(self.entity).insert((
        Stackable::new(stack_type),
        StackCount::new(count),
    ));
    self
}
```

### 3. Update Gold Nugget Prefab
```rust
pub fn spawn_gold_nugget(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(28, Palette::Yellow, Palette::White, Layer::Objects)
        .with_label("{Y-Y-Y-Y-Y-Y-Y-Y-Y-Y-Y-W scrollf|Gold Nugget}")
        .with_item(0.5)
        .with_needs_stable_id()
        .with_stackable(StackableType::GoldNugget, 1)
        .build();
}
```

### 4. Modify PickupItemAction
Key changes:
- Check if item has `Stackable` component
- Find existing stack of same type in inventory
- If found, merge stacks (handling overflow)
- If not found or non-stackable, use existing pickup logic
- Despawn picked up entity if fully merged into existing stack

Helper function needed:
```rust
fn find_existing_stack(
    world: &World, 
    item_ids: &[u64], 
    stack_type: StackableType
) -> Option<Entity>
```

### 5. Modify DropItemAction
- Check for `StackCount` component
- Item entity keeps its components when dropped
- Future: support partial stack drops

### 6. Update Inventory UI
In `state_inventory.rs`:
- Check for `StackCount` component when displaying items
- Format display as "Item Name x5" for stacks
- No changes to cursor handling needed

Query addition needed:
```rust
q_stack_counts: Query<&StackCount>
```

### 7. Update TransferItemAction
- Check if item being transferred is stackable
- Handle merging with existing stacks in destination inventory
- Support overflow back to source inventory

### 8. Register Components
In `main.rs`, add to the component registry:
```rust
reg.register::<Stackable>();
reg.register::<StackCount>();
```

## Benefits of This Approach

1. **Minimal Changes**: Inventory struct remains unchanged
2. **Entity Consistency**: Stacks are real entities with all normal properties
3. **Save Compatibility**: Just new components, existing saves work
4. **Leverages Existing Systems**: Glyph, Label, Item components work as-is
5. **Simple UI Updates**: Just check for StackCount when displaying
6. **Natural Serialization**: Components serialize/deserialize automatically
7. **Flexible**: Easy to add stack-specific behaviors

## Testing Plan

1. Spawn multiple gold nuggets
2. Pick up gold nuggets to test stacking
3. Verify inventory UI shows "Gold Nugget x5"
4. Test dropping stacked items
5. Test transferring stacks between containers
6. Test save/load with stacked items
7. Test stack overflow (picking up when near 99)

## Future Enhancements

- Partial stack drops (drop specific amount)
- Stack splitting UI
- Different max stack sizes per item type (if needed)
- Auto-stack when items enter inventory
- Stack merging when organizing inventory