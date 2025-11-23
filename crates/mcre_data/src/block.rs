use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub id: u16,
    pub name: String,         // "oak_planks"
    pub display_name: String, // "Oak Planks",
    pub default_state: u16,
    pub min_state_id: u16,
    pub max_state_id: u16,
    pub states: Vec<BlockStateField>,
}

#[derive(Serialize, Deserialize)]
pub struct BlockStateField {
    pub name: String,
    #[serde(flatten)]
    pub values: BlockStateFieldValues,
}

#[derive(Serialize, Deserialize)]
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

impl Block {
    pub async fn all() -> io::Result<Vec<Self>> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let block_data_path = root.join("blocks.json");
        let block_data_json = fs::read_to_string(block_data_path).await?;
        let block_data: Vec<Self> = serde_json::from_str(&block_data_json)?;

        Ok(block_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::block::Block;

    #[tokio::test]
    async fn test_block_data_load() {
        let blocks = Block::all().await.unwrap();
        assert!(!blocks.is_empty());
    }
}
