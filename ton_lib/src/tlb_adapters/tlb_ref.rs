use std::marker::PhantomData;
use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::error::TonlibError;
use ton_lib_core::traits::tlb::TLB;

/// TLBRef - allows to save object in a reference cell ( ^X).
/// use `#[tlb_derive(adapter="TLBRef")]` to apply it using TLBDerive macro
#[derive(Debug, Clone, PartialEq)]
pub struct TLBRef<T: TLB>(PhantomData<T>);

impl<T: TLB> TLBRef<T> {
    pub fn new() -> Self { TLBRef(PhantomData) }
    pub fn read(&self, parser: &mut CellParser) -> Result<T, TonlibError> { T::from_cell(parser.read_next_ref()?) }
    pub fn write(&self, builder: &mut CellBuilder, val: &T) -> Result<(), TonlibError> {
        builder.write_ref(val.to_cell_ref()?)
    }
}

impl<T: TLB> Default for TLBRef<T> {
    fn default() -> Self { Self::new() }
}

mod tests {
    use super::*;
    use ton_lib_core::TLBDerive;

    #[derive(TLBDerive, PartialEq, Debug)]
    struct TestStruct {
        #[tlb_derive(adapter = "TLBRef::<u8>::new()")]
        pub a: u8,
        #[tlb_derive(adapter = "TLBRef")]
        pub b: u8,
    }

    #[test]
    fn test_tlb_ref_opt_derive() -> anyhow::Result<()> {
        let expected = TestStruct { a: 255, b: 255 };
        let cell = expected.to_cell()?;
        assert_eq!(cell.refs.len(), 2);
        assert_eq!(cell.refs[0].data, vec![255]);
        assert_eq!(cell.refs[1].data, vec![255]);

        let parsed = TestStruct::from_cell(&cell)?;
        assert_eq!(parsed, expected);
        Ok(())
    }
}
