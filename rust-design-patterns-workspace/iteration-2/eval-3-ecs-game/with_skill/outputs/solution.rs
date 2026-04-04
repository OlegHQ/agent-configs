// Entity Component System (ECS) for game object management.
//
// Design decisions:
// - Components are plain data structs, stored in type-erased sparse maps keyed by EntityId.
// - Systems are trait objects that receive &mut World as context (no stored references).
// - Adding a new component = define a struct, no changes to existing code.
// - Adding a new system = implement the System trait, no changes to existing code.
// - Queries use generic methods on World to fetch component combinations by type.

use std::any::{Any, TypeId};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Newtype wrapper for entity identifiers -- prevents mixing up raw integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

/// Tracks the next available entity id and the set of live entities.
struct EntityAllocator {
    next_id: u64,
    alive: Vec<EntityId>,
}

impl EntityAllocator {
    fn new() -> Self {
        Self {
            next_id: 0,
            alive: Vec::new(),
        }
    }

    fn allocate(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.alive.push(id);
        id
    }

    fn deallocate(&mut self, entity: EntityId) {
        self.alive.retain(|&e| e != entity);
    }

    fn is_alive(&self, entity: EntityId) -> bool {
        self.alive.contains(&entity)
    }

    fn all_alive(&self) -> &[EntityId] {
        &self.alive
    }
}

// ---------------------------------------------------------------------------
// Component storage
// ---------------------------------------------------------------------------

/// Type-erased storage for one component type across all entities.
/// Internally a HashMap<EntityId, Box<dyn Any>> per component TypeId.
type ComponentMap = HashMap<EntityId, Box<dyn Any>>;

/// Central storage that maps each component type to its per-entity instances.
struct ComponentStore {
    stores: HashMap<TypeId, ComponentMap>,
}

impl ComponentStore {
    fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    fn insert<C: 'static>(&mut self, entity: EntityId, component: C) {
        self.stores
            .entry(TypeId::of::<C>())
            .or_default()
            .insert(entity, Box::new(component));
    }

    fn remove<C: 'static>(&mut self, entity: EntityId) {
        if let Some(map) = self.stores.get_mut(&TypeId::of::<C>()) {
            map.remove(&entity);
        }
    }

    fn remove_all(&mut self, entity: EntityId) {
        for map in self.stores.values_mut() {
            map.remove(&entity);
        }
    }

    fn get<C: 'static>(&self, entity: EntityId) -> Option<&C> {
        self.stores
            .get(&TypeId::of::<C>())?
            .get(&entity)?
            .downcast_ref::<C>()
    }

    fn get_mut<C: 'static>(&mut self, entity: EntityId) -> Option<&mut C> {
        self.stores
            .get_mut(&TypeId::of::<C>())?
            .get_mut(&entity)?
            .downcast_mut::<C>()
    }

    fn has<C: 'static>(&self, entity: EntityId) -> bool {
        self.stores
            .get(&TypeId::of::<C>())
            .map_or(false, |map| map.contains_key(&entity))
    }
}

// ---------------------------------------------------------------------------
// World -- owns entities and components, provides query helpers
// ---------------------------------------------------------------------------

pub struct World {
    entities: EntityAllocator,
    components: ComponentStore,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            components: ComponentStore::new(),
        }
    }

    /// Spawn a new entity and return its id.
    pub fn spawn(&mut self) -> EntityId {
        self.entities.allocate()
    }

    /// Destroy an entity and remove all its components.
    pub fn despawn(&mut self, entity: EntityId) {
        self.components.remove_all(entity);
        self.entities.deallocate(entity);
    }

    /// Attach a component to an entity (replaces if already present).
    pub fn insert<C: 'static>(&mut self, entity: EntityId, component: C) {
        self.components.insert(entity, component);
    }

    /// Remove a specific component type from an entity.
    pub fn remove_component<C: 'static>(&mut self, entity: EntityId) {
        self.components.remove::<C>(entity);
    }

    /// Get a shared reference to a component on an entity.
    pub fn get<C: 'static>(&self, entity: EntityId) -> Option<&C> {
        self.components.get::<C>(entity)
    }

    /// Get a mutable reference to a component on an entity.
    pub fn get_mut<C: 'static>(&mut self, entity: EntityId) -> Option<&mut C> {
        self.components.get_mut::<C>(entity)
    }

    /// Check whether an entity has a component.
    pub fn has<C: 'static>(&self, entity: EntityId) -> bool {
        self.components.has::<C>(entity)
    }

    /// Return all live entity ids.
    pub fn entities(&self) -> Vec<EntityId> {
        self.entities.all_alive().to_vec()
    }

    // -- Query helpers for common component combinations -----------------------

    /// Iterate over entities that possess both component A and component B.
    /// Returns entity ids; callers fetch components via `world.get::<T>()`.
    pub fn query_2<A: 'static, B: 'static>(&self) -> Vec<EntityId> {
        self.entities
            .all_alive()
            .iter()
            .copied()
            .filter(|&e| self.has::<A>(e) && self.has::<B>(e))
            .collect()
    }

    /// Query for entities with three required components.
    pub fn query_3<A: 'static, B: 'static, C: 'static>(&self) -> Vec<EntityId> {
        self.entities
            .all_alive()
            .iter()
            .copied()
            .filter(|&e| self.has::<A>(e) && self.has::<B>(e) && self.has::<C>(e))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// System trait -- open set, each system processes matching entities
// ---------------------------------------------------------------------------

/// Systems contain logic that operates on entities with specific component sets.
/// Context (`&mut World`) is passed into `run` -- systems do not store references
/// to the world (avoids borrow-checker fights and makes ordering explicit).
pub trait System {
    fn run(&mut self, world: &mut World);
}

// ---------------------------------------------------------------------------
// Components -- plain data, no behavior
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub radius: f32,
    pub damage_on_contact: i32,
}

#[derive(Debug, Clone)]
pub struct Ai {
    pub behavior: AiBehavior,
}

#[derive(Debug, Clone)]
pub enum AiBehavior {
    Idle,
    ChasePlayers,
    Patrol { waypoints: Vec<(f32, f32)>, current: usize },
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Moves every entity that has both Position and Velocity.
pub struct MovementSystem;

impl System for MovementSystem {
    fn run(&mut self, world: &mut World) {
        let entities = world.query_2::<Position, Velocity>();
        for entity in entities {
            // We must read velocity first, then mutate position, because
            // World only hands out one &mut at a time per call.
            let (dx, dy) = {
                let vel = world.get::<Velocity>(entity).unwrap();
                (vel.dx, vel.dy)
            };
            let pos = world.get_mut::<Position>(entity).unwrap();
            pos.x += dx;
            pos.y += dy;
        }
    }
}

/// Renders every entity that has both Position and Sprite.
pub struct RenderSystem;

impl System for RenderSystem {
    fn run(&mut self, world: &mut World) {
        let entities = world.query_2::<Position, Sprite>();
        for entity in entities {
            let pos = world.get::<Position>(entity).unwrap();
            let sprite = world.get::<Sprite>(entity).unwrap();
            println!(
                "Render '{texture}' ({w}x{h}) at ({x:.1}, {y:.1})",
                texture = sprite.texture,
                w = sprite.width,
                h = sprite.height,
                x = pos.x,
                y = pos.y,
            );
        }
    }
}

/// Applies contact damage between entities with Health+Collider based on proximity.
pub struct DamageSystem;

impl System for DamageSystem {
    fn run(&mut self, world: &mut World) {
        let entities = world.query_2::<Health, Collider>();
        // Collect positions and collider data for overlap detection.
        let mut entity_data: Vec<(EntityId, f32, f32, f32, i32)> = Vec::new();
        for &entity in &entities {
            if let (Some(pos), Some(col)) =
                (world.get::<Position>(entity), world.get::<Collider>(entity))
            {
                entity_data.push((entity, pos.x, pos.y, col.radius, col.damage_on_contact));
            }
        }

        // Detect overlapping pairs and accumulate damage.
        let mut damage_to_apply: Vec<(EntityId, i32)> = Vec::new();
        for i in 0..entity_data.len() {
            for j in (i + 1)..entity_data.len() {
                let (id_a, ax, ay, ar, a_dmg) = entity_data[i];
                let (id_b, bx, by, br, b_dmg) = entity_data[j];
                let dist_sq = (ax - bx).powi(2) + (ay - by).powi(2);
                let touch_dist = ar + br;
                if dist_sq <= touch_dist * touch_dist {
                    // Each entity takes damage from the other's collider.
                    damage_to_apply.push((id_a, b_dmg));
                    damage_to_apply.push((id_b, a_dmg));
                }
            }
        }

        for (entity, dmg) in damage_to_apply {
            if let Some(health) = world.get_mut::<Health>(entity) {
                health.current = (health.current - dmg).max(0);
                if health.current == 0 {
                    println!("Entity {:?} destroyed!", entity);
                }
            }
        }
    }
}

/// Ticks AI behavior for entities with Ai + Position.
pub struct AiSystem;

impl System for AiSystem {
    fn run(&mut self, world: &mut World) {
        let entities = world.query_2::<Ai, Position>();
        for entity in entities {
            // Read AI behavior, decide velocity changes.
            let behavior = {
                let ai = world.get::<Ai>(entity).unwrap();
                ai.behavior.clone()
            };
            match behavior {
                AiBehavior::Idle => {}
                AiBehavior::ChasePlayers => {
                    // Simplified: move toward origin as stand-in for "find nearest player".
                    let (tx, ty) = (0.0_f32, 0.0_f32);
                    let pos = world.get::<Position>(entity).unwrap();
                    let dir_x = (tx - pos.x).signum();
                    let dir_y = (ty - pos.y).signum();
                    if let Some(vel) = world.get_mut::<Velocity>(entity) {
                        vel.dx = dir_x * 1.0;
                        vel.dy = dir_y * 1.0;
                    }
                }
                AiBehavior::Patrol { waypoints, current } => {
                    if waypoints.is_empty() {
                        return;
                    }
                    let (wx, wy) = waypoints[current];
                    let pos = world.get::<Position>(entity).unwrap();
                    let dx = wx - pos.x;
                    let dy = wy - pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    // Advance to next waypoint if close enough.
                    let next = if dist < 1.0 {
                        (current + 1) % waypoints.len()
                    } else {
                        current
                    };

                    let speed = 2.0_f32;
                    if let Some(vel) = world.get_mut::<Velocity>(entity) {
                        if dist > 0.01 {
                            vel.dx = (dx / dist) * speed;
                            vel.dy = (dy / dist) * speed;
                        }
                    }

                    let ai = world.get_mut::<Ai>(entity).unwrap();
                    ai.behavior = AiBehavior::Patrol {
                        waypoints,
                        current: next,
                    };
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduler -- runs systems in a defined order each tick
// ---------------------------------------------------------------------------

pub struct Scheduler {
    systems: Vec<Box<dyn System>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Register a system. Systems execute in the order they are added.
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Run all systems once (one game tick).
    pub fn run_tick(&mut self, world: &mut World) {
        for system in self.systems.iter_mut() {
            system.run(world);
        }
    }
}

// ---------------------------------------------------------------------------
// Demonstration
// ---------------------------------------------------------------------------

fn main() {
    let mut world = World::new();
    let mut scheduler = Scheduler::new();

    // Register systems in execution order.
    scheduler.add_system(Box::new(AiSystem));
    scheduler.add_system(Box::new(MovementSystem));
    scheduler.add_system(Box::new(DamageSystem));
    scheduler.add_system(Box::new(RenderSystem));

    // --- Spawn entities with various component combinations ---

    // Player: Position + Velocity + Health + Sprite + Collider
    let player = world.spawn();
    world.insert(player, Position { x: 0.0, y: 0.0 });
    world.insert(player, Velocity { dx: 1.0, dy: 0.5 });
    world.insert(player, Health { current: 100, max: 100 });
    world.insert(
        player,
        Sprite {
            texture: "hero.png".into(),
            width: 32,
            height: 32,
        },
    );
    world.insert(
        player,
        Collider {
            radius: 16.0,
            damage_on_contact: 5,
        },
    );

    // Enemy: Position + Velocity + Health + Sprite + Collider + AI (chase)
    let enemy = world.spawn();
    world.insert(enemy, Position { x: 50.0, y: 50.0 });
    world.insert(enemy, Velocity { dx: 0.0, dy: 0.0 });
    world.insert(enemy, Health { current: 30, max: 30 });
    world.insert(
        enemy,
        Sprite {
            texture: "goblin.png".into(),
            width: 24,
            height: 24,
        },
    );
    world.insert(
        enemy,
        Collider {
            radius: 12.0,
            damage_on_contact: 10,
        },
    );
    world.insert(
        enemy,
        Ai {
            behavior: AiBehavior::ChasePlayers,
        },
    );

    // Patrol guard: Position + Velocity + Health + Sprite + Collider + AI (patrol)
    let guard = world.spawn();
    world.insert(guard, Position { x: 100.0, y: 100.0 });
    world.insert(guard, Velocity { dx: 0.0, dy: 0.0 });
    world.insert(guard, Health { current: 50, max: 50 });
    world.insert(
        guard,
        Sprite {
            texture: "guard.png".into(),
            width: 28,
            height: 28,
        },
    );
    world.insert(
        guard,
        Collider {
            radius: 14.0,
            damage_on_contact: 15,
        },
    );
    world.insert(
        guard,
        Ai {
            behavior: AiBehavior::Patrol {
                waypoints: vec![(80.0, 80.0), (120.0, 80.0), (120.0, 120.0), (80.0, 120.0)],
                current: 0,
            },
        },
    );

    // Static decoration: Position + Sprite only (no movement, no collision)
    let tree = world.spawn();
    world.insert(tree, Position { x: 25.0, y: 25.0 });
    world.insert(
        tree,
        Sprite {
            texture: "tree.png".into(),
            width: 48,
            height: 64,
        },
    );

    // --- Run a few game ticks ---
    println!("=== ECS Game Loop ===\n");
    for tick in 1..=3 {
        println!("--- Tick {tick} ---");
        scheduler.run_tick(&mut world);

        // Print health status.
        for &entity in &[player, enemy, guard] {
            if let Some(health) = world.get::<Health>(entity) {
                let pos = world.get::<Position>(entity).unwrap();
                println!(
                    "  {:?} at ({:.1}, {:.1}) HP: {}/{}",
                    entity, pos.x, pos.y, health.current, health.max
                );
            }
        }
        println!();
    }

    // --- Demonstrate adding a new component type at runtime ---
    // No existing code needs to change -- just define a struct and insert it.
    #[derive(Debug)]
    struct Inventory {
        items: Vec<String>,
    }
    let inv = Inventory {
        items: vec!["Sword".into(), "Shield".into()],
    };
    world.insert(player, inv);
    println!(
        "Player inventory: {:?}",
        world.get::<Inventory>(player).unwrap()
    );

    // --- Demonstrate despawning ---
    world.despawn(enemy);
    println!(
        "Enemy alive after despawn: {}",
        world.entities().contains(&enemy)
    );
}
