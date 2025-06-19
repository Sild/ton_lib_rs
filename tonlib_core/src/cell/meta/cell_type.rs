use crate::error::TonlibError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryRef,
    MerkleProof,
    MerkleUpdate,
}

impl CellType {
    // https://docs.ton.org/v3/documentation/data-formats/tlb/exotic-cells
    pub fn new_exotic(byte: u8) -> Result<CellType, TonlibError> {
        let cell_type = match byte {
            0x01 => Self::PrunedBranch,
            0x02 => Self::LibraryRef,
            0x03 => Self::MerkleProof,
            0x04 => Self::MerkleUpdate,
            _ => return Err(TonlibError::BOCWrongCellTypeTag(byte)),
        };
        Ok(cell_type)
    }

    pub fn is_exotic(&self) -> bool { self != &CellType::Ordinary }
}
