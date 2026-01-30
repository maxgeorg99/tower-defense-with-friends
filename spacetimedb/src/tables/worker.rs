// =============================================================================
// Worker & Resource Tables
// =============================================================================
//
// Workers harvest resources and bring them back to buildings.
// Resources include: Wood (trees), Gold (mines), Meat (sheep)
//
// =============================================================================

use spacetimedb::SpacetimeType;

// =============================================================================
// Resource Types
// =============================================================================

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourceType {
    #[default]
    Wood,
    Gold,
    Meat,
}

// =============================================================================
// Worker State Machine
// =============================================================================

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkerState {
    #[default]
    Idle,
    MovingToResource,
    Harvesting,
    ReturningWithResource,
}

// =============================================================================
// Worker Component
// =============================================================================

/// Worker entity component - harvests resources
#[spacetimedb::table(name = worker_component, public)]
#[derive(Clone)]
pub struct WorkerComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Movement speed
    pub speed: f32,

    /// Home building entity ID (returns here to deposit)
    pub home_building_id: u64,

    /// Current state in the state machine
    pub state: WorkerState,

    /// Resource currently carrying (if any)
    pub carrying_resource: Option<ResourceType>,

    /// Target resource node entity ID (when harvesting)
    pub target_resource_id: Option<u64>,

    /// Target position (for movement)
    pub target_x: f32,
    pub target_y: f32,

    /// Harvest progress timer (seconds)
    pub harvest_progress: f32,
}

// =============================================================================
// Resource Node Component
// =============================================================================

/// Resource node entity component (tree, gold mine, sheep)
#[spacetimedb::table(name = resource_node_component, public)]
#[derive(Clone)]
pub struct ResourceNodeComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Type of resource this node provides
    pub resource_type: ResourceType,

    /// Remaining harvests before depleted
    pub remaining: i32,

    /// Whether this node is depleted
    pub depleted: bool,
}

// =============================================================================
// Worker Building Component
// =============================================================================

/// Building that spawns and houses workers
#[spacetimedb::table(name = worker_building_component, public)]
#[derive(Clone)]
pub struct WorkerBuildingComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Maximum workers this building can support
    pub worker_capacity: i32,

    /// Current number of workers
    pub current_workers: i32,

    /// Time until next worker can be spawned (if auto-spawn)
    pub spawn_cooldown: f32,
}

// =============================================================================
// Constants
// =============================================================================

/// Time to harvest one resource (seconds)
pub const HARVEST_TIME: f32 = 2.0;

/// Default worker speed
pub const WORKER_SPEED: f32 = 50.0;

/// Cost to buy a new worker (gold)
pub const WORKER_COST: i32 = 25;

/// Resources per harvest
pub const RESOURCES_PER_HARVEST: i32 = 1;
