use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonlibError;
use crate::types::tlb::TLB;

impl TLB for bool {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> { parser.read_bit() }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> { builder.write_bit(*self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_cell::TonCell;

    #[test]
    fn test_bool() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        true.write(&mut builder)?;
        false.write(&mut builder)?;
        let cell = builder.build()?;
        assert_eq!(cell.data_bits_len, 2);
        assert_eq!(cell.data, vec![0b10000000]);
        Ok(())
    }
}
