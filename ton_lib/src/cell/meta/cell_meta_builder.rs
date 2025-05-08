use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::meta::cell_type::CellType;
use crate::cell::meta::level_mask::LevelMask;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use bitstream_io::{BigEndian, BitWrite, BitWriter, ByteRead, ByteReader};
use sha2::{Digest, Sha256};
use std::io::Cursor;

pub(super) struct CellMetaBuilder<'a> {
    pub(super) cell_type: CellType,
    pub(super) data: &'a [u8],
    pub(super) data_bits_len: usize,
    pub(super) refs: &'a [TonCellRef],
}

type CellBitWriter = BitWriter<Vec<u8>, BigEndian>;
struct Pruned {
    hash: TonHash,
    depth: u16,
}

impl<'a> CellMetaBuilder<'a> {
    pub(super) fn new(cell_type: CellType, data: &'a [u8], data_bits_len: usize, refs: &'a [TonCellRef]) -> Self {
        Self {
            cell_type,
            data,
            data_bits_len,
            refs,
        }
    }

    pub(super) fn validate(&self) -> Result<(), TonlibError> {
        match self.cell_type {
            CellType::Ordinary => self.validate_ordinary(), // guaranteed by builder
            CellType::PrunedBranch => self.validate_pruned(),
            CellType::Library => self.validate_library(),
            CellType::MerkleProof => self.validate_merkle_proof(),
            CellType::MerkleUpdate => self.validate_merkle_update(),
        }
    }

    pub(super) fn calc_level_mask(&self) -> LevelMask {
        match self.cell_type {
            CellType::Ordinary => self.calc_level_mask_ordinary(),
            CellType::PrunedBranch => self.calc_level_mask_pruned(),
            CellType::Library => LevelMask::new(0),
            CellType::MerkleProof => self.refs[0].meta.level_mask >> 1,
            CellType::MerkleUpdate => self.calc_level_mask_merkle_update(),
        }
    }

    fn validate_ordinary(&self) -> Result<(), TonlibError> {
        if self.data_bits_len > TonCell::MAX_DATA_BITS_LEN {
            return Err(TonlibError::BuilderMeta("Ordinary cell data bits length is too big".to_owned()));
        }
        Ok(())
    }

    fn validate_pruned(&self) -> Result<(), TonlibError> {
        if !self.refs.is_empty() {
            return Err(TonlibError::BuilderMeta("Pruned cell can't have refs".to_owned()));
        }
        if self.data_bits_len < 16 {
            return Err(TonlibError::BuilderMeta("PrunedBranch require at least 16 bits data".to_owned()));
        }

        if self.is_config_proof() {
            return Ok(());
        }

        let level_mask = self.calc_level_mask_pruned();

        if level_mask == LevelMask::MIN_LEVEL || level_mask > LevelMask::MAX_LEVEL {
            let err_msg = format!("Pruned Branch cell level must in range [1, 3] (got {level_mask})");
            return Err(TonlibError::BuilderMeta(err_msg));
        }

        let expected_size = (2 + level_mask.apply(level_mask.level() - 1u8).hash_count()
            * (TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES))
            * 8;

        if self.data_bits_len != expected_size {
            let err_msg = format!("PrunedBranch must have exactly {expected_size} bits, got {}", self.data_bits_len);
            return Err(TonlibError::BuilderMeta(err_msg));
        }

        Ok(())
    }

    fn validate_library(&self) -> Result<(), TonlibError> {
        const LIB_CELL_BITS_LEN: usize = (1 + TonHash::BYTES_LEN) * 8;

        if self.data_bits_len != LIB_CELL_BITS_LEN {
            let err_msg =
                format!("Library cell must have exactly {LIB_CELL_BITS_LEN} bits, got {}", self.data_bits_len);
            return Err(TonlibError::BuilderMeta(err_msg));
        }

        Ok(())
    }

    fn validate_merkle_proof(&self) -> Result<(), TonlibError> {
        // type + hash + depth
        const MERKLE_PROOF_BITS_LEN: usize = (1 + TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES) * 8;

        if self.data_bits_len != MERKLE_PROOF_BITS_LEN {
            let err_msg =
                format!("MerkleProof must have exactly {MERKLE_PROOF_BITS_LEN} bits, got {}", self.data_bits_len);
            return Err(TonlibError::BuilderMeta(err_msg));
        }

        if self.refs.len() != 1 {
            return Err(TonlibError::BuilderMeta("Merkle Proof cell must have exactly 1 ref".to_owned()));
        }

        let mut data_slice = &self.data[1..];
        let _proof_hash = TonHash::from_bytes(&data_slice[..TonHash::BYTES_LEN])
            .map_err(|err| TonlibError::BuilderMeta(format!("Can't get proof hash bytes from cell data: {err}")))?;

        data_slice = &data_slice[TonHash::BYTES_LEN..];
        let _proof_depth = u16::from_be_bytes(data_slice[..CellMeta::DEPTH_BYTES].try_into().unwrap());
        log::warn!("validate_merkle_proof is not implemented yet!");
        Ok(())
    }

    fn validate_merkle_update(&self) -> Result<(), TonlibError> {
        // type + hash + hash + depth + depth
        // const MERKLE_UPDATE_BITS_LEN: usize = 8 + (2 * (256 + 16));
        log::error!("validate_merkle_proof is not implemented yet");
        Ok(())
        // todo!();
    }

    fn calc_level_mask_ordinary(&self) -> LevelMask {
        let mut mask = LevelMask::new(0);
        for cell_ref in self.refs {
            mask |= cell_ref.meta.level_mask;
        }
        mask
    }

    fn calc_level_mask_pruned(&self) -> LevelMask {
        match self.is_config_proof() {
            true => LevelMask::new(1),
            false => LevelMask::new(self.data[1]),
        }
    }

    fn calc_level_mask_merkle_update(&self) -> LevelMask {
        let refs_lm = self.refs[0].meta.level_mask | self.refs[1].meta.level_mask;
        refs_lm >> 1
    }

    fn is_config_proof(&self) -> bool {
        const CONFIG_PROOF_DATA_LEN_BITS: usize = 200;
        self.cell_type == CellType::PrunedBranch && self.data_bits_len == CONFIG_PROOF_DATA_LEN_BITS
    }

    /// This function replicates unknown logic of resolving cell data
    /// https://github.com/ton-blockchain/ton/blob/24dc184a2ea67f9c47042b4104bbb4d82289fac1/crypto/vm/cells/DataCell.cpp#L214
    pub(super) fn calc_hashes_and_depths(
        &self,
        level_mask: LevelMask,
    ) -> Result<([TonHash; 4], [u16; 4]), TonlibError> {
        let hash_count = if self.cell_type == CellType::PrunedBranch {
            1
        } else {
            level_mask.hash_count()
        };

        let total_hash_count = level_mask.hash_count();
        let hash_i_offset = total_hash_count - hash_count;

        let mut hashes = [TonHash::ZERO; 4];
        let mut depths = [0; 4];

        // Iterate through significant levels
        for (hash_pos, level_pos) in (0..=level_mask.level()).filter(|&i| level_mask.is_significant(i)).enumerate() {
            if hash_pos < hash_i_offset {
                continue;
            }

            let (cur_data, cur_bit_len) = if hash_pos == hash_i_offset {
                (self.data, self.data_bits_len)
            } else {
                let prev_hash = &hashes[hash_pos - hash_i_offset - 1];
                (prev_hash.as_slice(), 256)
            };

            // Calculate Depth
            let depth = if self.refs.is_empty() {
                0
            } else {
                let mut max_ref_depth = 0;
                for cell_ref in self.refs {
                    let ref_depth = self.get_ref_depth(cell_ref, level_pos);
                    max_ref_depth = max_ref_depth.max(ref_depth);
                }
                max_ref_depth + 1
            };

            // Calculate Hash
            let repr = self.get_repr_for_data(cur_data, cur_bit_len, level_mask, level_pos)?;
            let hash = TonHash::from_bytes(&Sha256::new_with_prefix(repr).finalize())?;
            hashes[hash_pos] = hash;
            depths[hash_pos] = depth;
        }

        self.resolve_hashes_and_depths(hashes, depths, level_mask)
    }

    fn get_repr_for_data(
        &self,
        cur_data: &[u8],
        cur_data_bits_len: usize,
        level_mask: LevelMask,
        level: u8,
    ) -> Result<Vec<u8>, TonlibError> {
        // descriptors + data + (hash + depth) * refs_count
        let buffer_len = 2 + cur_data.len() + (32 + 2) * self.refs.len();

        let mut writer = BitWriter::endian(Vec::with_capacity(buffer_len), BigEndian);
        let d1 = self.get_refs_descriptor(level_mask.apply(level));
        let d2 = get_bits_descriptor(self.data_bits_len);

        // Write descriptors
        writer.write_var(8, d1)?;
        writer.write_var(8, d2)?;
        // Write main data
        write_data(&mut writer, cur_data, cur_data_bits_len)?;
        // Write ref data
        self.write_ref_depths(&mut writer, level)?;
        self.write_ref_hashes(&mut writer, level)?;

        let result = writer
            .writer()
            .ok_or_else(|| TonlibError::BuilderMeta("Stream for cell repr is not byte-aligned".to_string()))?
            .to_vec();

        Ok(result)
    }

    /// Calculates d1 descriptor for cell
    /// See https://docs.ton.org/tvm.pdf 3.1.4 for details
    fn get_refs_descriptor<L: Into<u8>>(&self, level_mask: L) -> u8 {
        let cell_type_var = (self.cell_type != CellType::Ordinary) as u8;
        self.refs.len() as u8 + 8 * cell_type_var + level_mask.into() * 32
    }

    fn write_ref_hashes(&self, writer: &mut CellBitWriter, level: u8) -> Result<(), TonlibError> {
        for cell_ref in self.refs {
            let ref_hash = self.get_ref_hash(cell_ref, level);
            writer.write_bytes(ref_hash.as_slice())?;
        }

        Ok(())
    }

    fn write_ref_depths(&self, writer: &mut CellBitWriter, level: u8) -> Result<(), TonlibError> {
        for cell_ref in self.refs {
            let ref_depth = self.get_ref_depth(cell_ref, level);
            writer.write_var(8, ref_depth / 256)?;
            writer.write_var(8, ref_depth % 256)?;
        }
        Ok(())
    }

    fn resolve_hashes_and_depths(
        &self,
        hashes: [TonHash; 4],
        depths: [u16; 4],
        level_mask: LevelMask,
    ) -> Result<([TonHash; 4], [u16; 4]), TonlibError> {
        let mut resolved_hashes = [TonHash::ZERO; 4];
        let mut resolved_depths = [0; 4];

        for i in 0..4 {
            let hash_index = level_mask.apply(i).hash_index();

            let (hash, depth) = match self.cell_type {
                CellType::PrunedBranch => {
                    let this_hash_index = level_mask.hash_index();
                    if hash_index != this_hash_index {
                        let pruned = self.calc_pruned_hash_depth(level_mask)?;
                        (pruned[hash_index].hash.clone(), pruned[hash_index].depth)
                    } else {
                        (hashes[0].clone(), depths[0])
                    }
                }
                _ => (hashes[hash_index].clone(), depths[hash_index]),
            };

            resolved_hashes[i as usize] = hash;
            resolved_depths[i as usize] = depth;
        }

        Ok((resolved_hashes, resolved_depths))
    }

    fn get_ref_depth(&self, cell_ref: &TonCell, level: u8) -> u16 {
        let extra_level = matches!(self.cell_type, CellType::MerkleProof | CellType::MerkleUpdate) as usize;
        cell_ref.meta.depths[level as usize + extra_level]
    }

    fn get_ref_hash(&self, cell_ref: &TonCell, level: u8) -> TonHash {
        let extra_level = matches!(self.cell_type, CellType::MerkleProof | CellType::MerkleUpdate) as usize;
        cell_ref.meta.hashes[level as usize + extra_level].clone()
    }

    fn calc_pruned_hash_depth(&self, level_mask: LevelMask) -> Result<Vec<Pruned>, TonlibError> {
        let current_index = if self.is_config_proof() { 1 } else { 2 };

        let cursor = Cursor::new(&self.data[current_index..]);
        let mut reader = ByteReader::endian(cursor, BigEndian);

        let level = level_mask.level() as usize;
        let hashes = (0..level).map(|_| reader.read::<[u8; TonHash::BYTES_LEN]>()).collect::<Result<Vec<_>, _>>()?;
        let depths = (0..level).map(|_| reader.read::<u16>()).collect::<Result<Vec<_>, _>>()?;

        let result = hashes
            .into_iter()
            .zip(depths)
            .map(|(hash, depth)| Pruned {
                hash: hash.into(),
                depth,
            })
            .collect();

        Ok(result)
    }
}

/// Calculates d2 descriptor for cell
/// See https://docs.ton.org/tvm.pdf 3.1.4 for details
fn get_bits_descriptor(data_bits_len: usize) -> u8 { (data_bits_len / 8 + data_bits_len.div_ceil(8)) as u8 }

fn write_data(writer: &mut CellBitWriter, data: &[u8], bit_len: usize) -> Result<(), TonlibError> {
    let data_len = data.len();
    let rest_bits = bit_len % 8;
    let full_bytes = rest_bits == 0;

    if !full_bytes {
        writer.write_bytes(&data[..data_len - 1])?;
        let last_byte = data[data_len - 1];
        let l = last_byte | (1 << (8 - rest_bits - 1));
        writer.write_var(8, l)?;
    } else {
        writer.write_bytes(data)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn empty_cell_ref() -> TonCellRef { TonCell::EMPTY.into_ref() }

    #[test]
    fn test_refs_descriptor_d1() {
        let meta_builder = CellMetaBuilder::new(CellType::Ordinary, &[], 0, &[]);
        assert_eq!(meta_builder.get_refs_descriptor(0), 0);
        assert_eq!(meta_builder.get_refs_descriptor(3), 96);

        let refs = [empty_cell_ref()];
        let meta_builder = CellMetaBuilder::new(CellType::Ordinary, &[], 0, &refs);
        assert_eq!(meta_builder.get_refs_descriptor(3), 97);

        let refs = [empty_cell_ref(), empty_cell_ref()];
        let meta_builder = CellMetaBuilder::new(CellType::Ordinary, &[], 0, &refs);
        assert_eq!(meta_builder.get_refs_descriptor(3), 98);
    }

    #[test]
    fn test_bits_descriptor_d2() {
        assert_eq!(get_bits_descriptor(0), 0);
        assert_eq!(get_bits_descriptor(1023), 255);
    }

    #[test]
    fn test_hashes_and_depths() -> anyhow::Result<()> {
        let meta_builder = CellMetaBuilder::new(CellType::Ordinary, &[], 0, &[]);
        let level_mask = LevelMask::new(0);
        let (hashes, depths) = meta_builder.calc_hashes_and_depths(level_mask)?;

        for i in 0..4 {
            assert_eq!(hashes[i], TonHash::EMPTY_CELL_HASH);
            assert_eq!(depths[i], 0);
        }
        Ok(())
    }
}
