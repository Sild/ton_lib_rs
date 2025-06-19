use crate::cell::meta::cell_meta_builder::CellMetaBuilder;
use crate::cell::meta::cell_type::CellType;
use crate::cell::meta::level_mask::LevelMask;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::cell::ton_hash::TonHash;
use crate::error::TonlibError;

#[derive(Debug, Clone, PartialEq)]
pub struct CellMeta {
    pub cell_type: CellType,
    pub level_mask: LevelMask,
    pub depths: [u16; 4],
    pub hashes: [TonHash; 4],
}

impl CellMeta {
    pub const DEPTH_BYTES: usize = 2;

    pub const EMPTY_CELL_META: CellMeta = CellMeta {
        cell_type: CellType::Ordinary,
        level_mask: LevelMask::new(0),
        depths: [0; 4],
        hashes: [TonCell::EMPTY_CELL_HASH; 4],
    };

    pub fn new(
        cell_type: CellType,
        data: &[u8],
        data_bits_len: usize,
        refs: &[TonCellRef],
    ) -> Result<Self, TonlibError> {
        let meta_builder = CellMetaBuilder::new(cell_type, data, data_bits_len, refs);

        // just don't look inside
        meta_builder.validate()?;
        let level_mask = meta_builder.calc_level_mask();
        let (hashes, depths) = meta_builder.calc_hashes_and_depths(level_mask)?;

        let meta = Self {
            cell_type,
            level_mask,
            depths,
            hashes,
        };
        Ok(meta)
    }
}
