use serde::Serialize;

#[derive(Serialize)]
pub struct Block {
    pub id: u16,
    pub name: String,         // "oak_planks"
    pub display_name: String, // "Oak Planks",
    pub default_state: u16,
    pub min_state_id: u16,
    pub max_state_id: u16,
    pub states: Vec<BlockStateField>,
}

#[derive(Serialize)]
pub struct BlockStateField {
    pub name: String,
    #[serde(flatten)]
    pub values: BlockStateFieldValues,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BlockStateFieldValues {
    Bool,
    Enum {
        enum_name: String,
        values: Vec<String>,
    },
    Int {
        // inclusive range, [min, max]
        min: u8,
        max: u8,
    },
}
