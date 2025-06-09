use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCell;
use crate::errors::TonlibError;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapter;
use crate::types::tlb::TLB;
use std::marker::PhantomData;

pub struct BinTree<VA: DictValAdapter<T>, T: TLB>(PhantomData<(VA, T)>);

impl<VA: DictValAdapter<T>, T: TLB> BinTree<VA, T> {
    pub fn new() -> Self { Self(PhantomData) }

    pub fn read(&self, parser: &mut CellParser) -> Result<Vec<T>, TonlibError> {
        if !parser.read_bit()? {
            return Ok(vec![VA::read(parser)?]);
        }
        let mut left = self.read(&mut parser.read_next_ref()?.parser())?;
        let right = self.read(&mut parser.read_next_ref()?.parser())?;
        left.extend(right);
        Ok(left)
    }

    pub fn write(&self, builder: &mut CellBuilder, data: &[T]) -> Result<(), TonlibError> {
        if data.len() == 1 {
            builder.write_bit(false)?;
            return VA::write(builder, &data[0]);
        }
        builder.write_bit(true)?;

        let mut left_builder = TonCell::builder();
        self.write(&mut left_builder, &data[0..data.len() / 2])?;
        builder.write_ref(left_builder.build()?.into_ref())?;

        let mut right_builder = TonCell::builder();
        self.write(&mut right_builder, &data[data.len() / 2..])?;
        builder.write_ref(right_builder.build()?.into_ref())?;
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
        let data = vec![1, 2, 3, 4, 5, 6];
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
