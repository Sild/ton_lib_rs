use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::error::TonlibError;
use crate::traits::tlb::TLB;

// https://github.com/ton-blockchain/ton/blob/2a68c8610bf28b43b2019a479a70d0606c2a0aa1/crypto/block/block.tlb#L11
#[derive(Clone, Debug, PartialEq)]
pub enum TLBEither<L, R> {
    Left(L),
    Right(R),
}

impl<L: TLB, R: TLB> TLB for TLBEither<L, R> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        match parser.read_bit()? {
            false => Ok(Self::Left(L::read(parser)?)),
            true => Ok(Self::Right(R::read(parser)?)),
        }
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> {
        match self {
            TLBEither::Left(left) => {
                builder.write_bit(false)?;
                left.write(builder)
            }
            TLBEither::Right(right) => {
                builder.write_bit(true)?;
                right.write(builder)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonCell;
    use tokio_test::assert_ok;
    use ton_lib_macros::TLBDerive;

    #[derive(Debug, PartialEq, TLBDerive)]
    struct TestType1(i32);

    #[derive(Debug, PartialEq, TLBDerive)]
    struct TestType2(i64);

    #[test]
    fn test_either() -> anyhow::Result<()> {
        let obj1: TLBEither<TestType1, TestType2> = TLBEither::Left(TestType1(1));
        let obj2: TLBEither<TestType1, TestType2> = TLBEither::Right(TestType2(2));
        let mut builder = TonCell::builder();
        obj1.write(&mut builder)?;
        obj2.write(&mut builder)?;
        let cell = builder.build()?;
        let mut parser = cell.parser();
        let parsed_obj1 = TLB::read(&mut parser)?;
        let parsed_obj2 = TLB::read(&mut parser)?;
        assert_eq!(obj1, parsed_obj1);
        assert_eq!(obj2, parsed_obj2);

        // check raw data
        let mut parser = cell.parser();
        assert!(!parser.read_bit()?);
        assert_ok!(parser.read_bits(32)); // skipping
        assert!(parser.read_bit()?);
        Ok(())
    }
}
