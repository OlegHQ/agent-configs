use std::any::{Any, TypeId};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Component trait + concrete components
// ---------------------------------------------------------------------------

/// Marker trait for all components. Every component must be 'static so we can
/// use TypeId for storage keys.
trait Component: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Blanket: every 'static type that opts in via `impl Component` automatically
/// gets the Any bridging for free.
macro_rules! impl_component {
    ($t:ty) => {
        impl Component for $t {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
        }
    };
}

#[derive(Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
}
impl_component!(Position);

#[derive(Debug, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}
impl_component!(Velocity);

#[derive(Debug, Clone)]
struct Health {
    current: f32,
    max: f32,
}
impl_component!(Health);

#[derive(Debug, Clone)]
struct Sprite {
    texture_id: u32,
    width: u32,
    height: u32,
}
impl_component!(Sprite);

#[derive(Debug, Clone)]
struct Collider {
    radius: f32,
    damage: f32,
}
impl_component!(Collider);

#[derive(Debug, Clone)]
struct AI {
    behaviour: AIBehaviour,
}
impl_component!(AI);

#[derive(Debug, Clone)]
enum AIBehaviour {
    ChasePlayer,
    Patrol,
    Idle,
}

// ---------------------------------------------------------------------------
// Entity ID
// ---------------------------------------------------------------------------

/// A lightweight handle for an entity — just an index + a generation counter
/// so we can detect stale handles after an entity is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Entity {
    id: usize,
    generation: u32,
}

// ---------------------------------------------------------------------------
// World — the core ECS storage
// ---------------------------------------------------------------------------

struct World {
    /// Per-entity generation counter (used to validate Entity handles).
    generations: Vec<u32>,
    /// Bit that says whether slot `i` is alive.
    alive: Vec<bool>,
    /// Sparse component storage: TypeId -> (entity-index -> Box<dyn Component>).
    components: HashMap<TypeId, HashMap<usize, Box<dyn Component>>>,
    /// Recycled entity slots.
    free_list: Vec<usize>,
}

impl World {
    fn new() -> Self {
        Self {
            generations: Vec::new(),
            alive: Vec::new(),
            components: HashMap::new(),
            free_list: Vec::new(),
        }
    }

    /// Spawn a new entity and return its handle.
    fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_list.pop() {
            self.alive[id] = true;
            Entity { id, generation: self.generations[id] }
        } else {
            let id = self.generations.len();
            self.generations.push(0);
            self.alive.push(true);
            Entity { id, generation: 0 }
        }
    }

    /// Destroy an entity, bumping its generation so stale handles are invalid.
    fn despawn(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        self.alive[entity.id] = false;
        self.generations[entity.id] += 1;
        // Remove all components for this entity.
        for storage in self.components.values_mut() {
            storage.remove(&entity.id);
        }
        self.free_list.push(entity.id);
    }

    fn is_alive(&self, entity: Entity) -> bool {
        entity.id < self.alive.len()
            && self.alive[entity.id]
            && self.generations[entity.id] == entity.generation
    }

    /// Attach a component to an entity (replaces if already present).
    fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        assert!(self.is_alive(entity), "cannot add component to dead entity");
        let storage = self.components.entry(TypeId::of::<T>()).or_default();
        storage.insert(entity.id, Box::new(component));
    }

    /// Get an immutable reference to a component on an entity.
    fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|s| s.get(&entity.id))
            .and_then(|c| c.as_any().downcast_ref::<T>())
    }

    /// Get a mutable reference to a component on an entity.
    fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.get_mut(&entity.id))
            .and_then(|c| c.as_any_mut().downcast_mut::<T>())
    }

    /// Check whether an entity has a given component type.
    fn has_component<T: Component + 'static>(&self, entity: Entity) -> bool {
        self.components
            .get(&TypeId::of::<T>())
            .map_or(false, |s| s.contains_key(&entity.id))
    }

    /// Return all living entities.
    fn entities(&self) -> Vec<Entity> {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, &a)| a)
            .map(|(id, _)| Entity { id, generation: self.generations[id] })
            .collect()
    }

    /// Return all living entities that have every component type in `required`.
    fn query(&self, required: &[TypeId]) -> Vec<Entity> {
        self.entities()
            .into_iter()
            .filter(|e| {
                required.iter().all(|tid| {
                    self.components
                        .get(tid)
                        .map_or(false, |s| s.contains_key(&e.id))
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// System trait + concrete systems
// ---------------------------------------------------------------------------

trait System {
    fn name(&self) -> &str;
    fn run(&self, world: &mut World);
}

// -- Movement: Position + Velocity ------------------------------------------

struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str { "MovementSystem" }

    fn run(&self, world: &mut World) {
        let entities = world.query(&[
            TypeId::of::<Position>(),
            TypeId::of::<Velocity>(),
        ]);

        for entity in entities {
            // Read velocity first (immutable borrow).
            let (dx, dy) = {
                let vel = world.get_component::<Velocity>(entity).unwrap();
                (vel.dx, vel.dy)
            };
            // Then mutate position.
            let pos = world.get_component_mut::<Position>(entity).unwrap();
            pos.x += dx;
            pos.y += dy;
        }
    }
}

// -- Render: Position + Sprite ----------------------------------------------

struct RenderSystem;

impl System for RenderSystem {
    fn name(&self) -> &str { "RenderSystem" }

    fn run(&self, world: &mut World) {
        let entities = world.query(&[
            TypeId::of::<Position>(),
            TypeId::of::<Sprite>(),
        ]);

        for entity in entities {
            let pos = world.get_component::<Position>(entity).unwrap();
            let sprite = world.get_component::<Sprite>(entity).unwrap();
            println!(
                "  [Render] entity {:?} | texture={} ({}x{}) at ({:.1}, {:.1})",
                entity.id, sprite.texture_id, sprite.width, sprite.height, pos.x, pos.y,
            );
        }
    }
}

// -- Damage: Health + Collider ----------------------------------------------

struct DamageSystem;

impl System for DamageSystem {
    fn name(&self) -> &str { "DamageSystem" }

    fn run(&self, world: &mut World) {
        let entities = world.query(&[
            TypeId::of::<Health>(),
            TypeId::of::<Collider>(),
        ]);

        for entity in entities {
            let damage = {
                let collider = world.get_component::<Collider>(entity).unwrap();
                collider.damage
            };
            let health = world.get_component_mut::<Health>(entity).unwrap();
            if damage > 0.0 {
                health.current = (health.current - damage).max(0.0);
                println!(
                    "  [Damage] entity {:?} took {:.1} damage -> HP {:.1}/{:.1}",
                    entity.id, damage, health.current, health.max,
                );
            }
        }
    }
}

// -- AI: Position + AI ------------------------------------------------------

struct AISystem;

impl System for AISystem {
    fn name(&self) -> &str { "AISystem" }

    fn run(&self, world: &mut World) {
        let entities = world.query(&[
            TypeId::of::<Position>(),
            TypeId::of::<AI>(),
        ]);

        for entity in entities {
            let pos = world.get_component::<Position>(entity).unwrap();
            let ai = world.get_component::<AI>(entity).unwrap();
            println!(
                "  [AI]     entity {:?} at ({:.1}, {:.1}) behaviour={:?}",
                entity.id, pos.x, pos.y, ai.behaviour,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduler — runs systems in a fixed order each tick
// ---------------------------------------------------------------------------

struct Scheduler {
    systems: Vec<Box<dyn System>>,
}

impl Scheduler {
    fn new() -> Self {
        Self { systems: Vec::new() }
    }

    fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    fn run_all(&self, world: &mut World) {
        for system in &self.systems {
            println!("--- {} ---", system.name());
            system.run(world);
        }
    }
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

fn main() {
    let mut world = World::new();
    let mut scheduler = Scheduler::new();

    // Register systems.
    scheduler.add_system(Box::new(AISystem));
    scheduler.add_system(Box::new(MovementSystem));
    scheduler.add_system(Box::new(DamageSystem));
    scheduler.add_system(Box::new(RenderSystem));

    // --- Spawn some entities ------------------------------------------------

    // Player: has Position, Velocity, Health, Sprite, Collider
    let player = world.spawn();
    world.add_component(player, Position { x: 0.0, y: 0.0 });
    world.add_component(player, Velocity { dx: 1.0, dy: 0.5 });
    world.add_component(player, Health { current: 100.0, max: 100.0 });
    world.add_component(player, Sprite { texture_id: 1, width: 32, height: 32 });
    world.add_component(player, Collider { radius: 16.0, damage: 0.0 });

    // Enemy: has Position, Velocity, Health, Sprite, Collider, AI
    let enemy = world.spawn();
    world.add_component(enemy, Position { x: 50.0, y: 30.0 });
    world.add_component(enemy, Velocity { dx: -0.5, dy: 0.0 });
    world.add_component(enemy, Health { current: 40.0, max: 40.0 });
    world.add_component(enemy, Sprite { texture_id: 2, width: 32, height: 32 });
    world.add_component(enemy, Collider { radius: 16.0, damage: 5.0 });
    world.add_component(enemy, AI { behaviour: AIBehaviour::ChasePlayer });

    // Decoration: only Position + Sprite (no physics, no health)
    let tree = world.spawn();
    world.add_component(tree, Position { x: 20.0, y: 10.0 });
    world.add_component(tree, Sprite { texture_id: 10, width: 16, height: 32 });

    // Patrol drone: Position + Velocity + AI (no sprite, invisible helper)
    let drone = world.spawn();
    world.add_component(drone, Position { x: 100.0, y: 100.0 });
    world.add_component(drone, Velocity { dx: 0.0, dy: -1.0 });
    world.add_component(drone, AI { behaviour: AIBehaviour::Patrol });

    // --- Simulate a few ticks ----------------------------------------------

    for tick in 0..3 {
        println!("\n========== TICK {} ==========", tick);
        scheduler.run_all(&mut world);
    }

    // --- Demonstrate despawn ------------------------------------------------

    println!("\n>>> Despawning enemy (entity {:?})", enemy.id);
    world.despawn(enemy);

    println!("\n========== TICK after despawn ==========");
    scheduler.run_all(&mut world);

    // --- Demonstrate adding a new component type at runtime -----------------

    // We can invent a brand-new component without touching World or System:
    #[derive(Debug)]
    struct Pickup { item_name: String }
    impl_component!(Pickup);

    let gem = world.spawn();
    world.add_component(gem, Position { x: 5.0, y: 5.0 });
    world.add_component(gem, Sprite { texture_id: 99, width: 8, height: 8 });
    world.add_component(gem, Pickup { item_name: "Ruby".into() });

    println!("\n>>> Spawned gem with custom Pickup component");
    println!(
        "  gem has Pickup? {} -> {:?}",
        world.has_component::<Pickup>(gem),
        world.get_component::<Pickup>(gem).map(|p| &p.item_name),
    );

    println!("\n========== TICK with gem ==========");
    scheduler.run_all(&mut world);
}
