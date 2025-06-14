use std::collections::HashMap;
use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCell;
use crate::errors::TonlibError;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapter;
use crate::types::tlb::TLB;
use std::marker::PhantomData;
use crate::types::tlb::block_tlb::block::shard_ident::ShardPfx;

// for now it's used only only with shard_pfx in keys
pub struct BinTree<VA: DictValAdapter<T>, T: TLB>(PhantomData<(VA, T)>);

impl<VA: DictValAdapter<T>, T: TLB> Default for BinTree<VA, T> {
    fn default() -> Self { Self::new() }
}

impl<VA: DictValAdapter<T>, T: TLB> BinTree<VA, T> {
    pub fn new() -> Self { Self(PhantomData) }

    pub fn read(&self, parser: &mut CellParser) -> Result<HashMap<ShardPfx, T>, TonlibError> {
        let mut val = HashMap::new();
        self.read_impl(parser, ShardPfx { value: 1, bits_len: 1}, &mut val)?;
        Ok(val)
    }
    
    fn read_impl(&self, parser: &mut CellParser, cur_key: ShardPfx, cur_val: &mut HashMap<ShardPfx, T>) -> Result<(), TonlibError> {
        if !parser.read_bit()? {
            cur_val.insert(cur_key, VA::read(parser)?);
            return Ok(());
        }
        self.read_impl(&mut parser.read_next_ref()?.parser(), cur_key << 1, cur_val)?;
        self.read_impl(&mut parser.read_next_ref()?.parser(), (cur_key << 1) + 1, cur_val)?;
        Ok(())
    }

    pub fn write(&self, builder: &mut CellBuilder, data: &HashMap<ShardPfx, T>) -> Result<(), TonlibError> {
        if data.is_empty() {
            return Err(TonlibError::TLBWrongData("BinTree can't be empty".to_string()));
        }
        self.write_impl(builder, 1, data)
    }
    
    fn write_impl(&self, builder: &mut CellBuilder, cur_key: ShardPfx, data: &HashMap<ShardPfx, T>) -> Result<(), TonlibError> {
        if let Some(val) = data.get(&cur_key) {
            builder.write_bit(false)?;
            return VA::write(builder, val);
        }
        builder.write_bit(true)?;

        let mut left_builder = TonCell::builder();
        self.write_impl(&mut left_builder, cur_key << 1, data)?;
        builder.write_ref(left_builder.build_ref()?)?;
        
        
        let mut right_builder = TonCell::builder();
        self.write_impl(&mut right_builder, (cur_key << 1) + 1, data)?;
        builder.write_ref(right_builder.build_ref()?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_cell::TonCell;
    use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterNum;

    #[test]
    fn test_bin_tree() -> anyhow::Result<()> {
        //                               * (1)
        //                 * (10)                       * (11) 
        //        *(100)        * [101]             * (110)      * [111]
        // * [1000]   * [1001]                 * [1100]  * [1101]
        let data = HashMap::from([
            (0b1000, 0),
            (0b1001, 1),
            (0b101, 2),
            (0b1100, 3),
            (0b1101, 4),
            (0b111, 5),
        ]);
        let mut builder = TonCell::builder();
        BinTree::<DictValAdapterNum<32>, u32>::new().write(&mut builder, &data)?;
        let cell = builder.build()?;
        println!("{:?}", cell);
        let mut parser = cell.parser();
        let parsed_data = BinTree::<DictValAdapterNum<32>, u32>::new().read(&mut parser)?;
        assert_eq!(data, parsed_data);
        Ok(())
    }
}
