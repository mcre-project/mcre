use indexmap::IndexMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct BlockState {
    pub id: u16,
    pub block_id: u16,
    pub block_name: String,
    // Light / Rendering
    // How much light the block emits (0–15).
    pub light_emission: u8,
    // If true, use the actual voxel shape for light occlusion (glass panes, fences).
    pub use_shape_for_light_occlusion: bool,
    // True if skylight passes through this block.
    pub propagates_skylight_down: bool,
    // How many light levels this block blocks (0–15).
    // Opposite of emission.
    pub light_block: u8,
    // Whether it is considered solid for rendering (solid faces culling).
    pub solid_render: bool,
    // Block Solidity + Physics
    // If this blockstate is "air" (no collision, no rendering, etc).
    pub is_air: bool,
    // Whether lava should ignite this block.
    pub ignited_by_lava: bool,
    // Whether this block can occlude other blocks (block light / face culling).
    pub can_occlude: bool,
    // TODO:
    // pub map_color: MapColor,
    pub is_randomly_ticking: bool,
    pub replaceable: bool,
    pub spawn_terrain_particles: bool,
    pub requires_correct_tool_for_drops: bool,
    pub destroy_speed: f32,
    pub offset_type: OffsetType,
    pub state_values: IndexMap<String, StateValue>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum StateValue {
    Int(u8),
    Bool(bool),
    String(String),
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OffsetType {
    None,
    XZ,
    XYZ,
}
