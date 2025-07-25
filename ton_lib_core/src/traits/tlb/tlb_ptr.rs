use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::error::TLCoreError;
use crate::traits::tlb::TLB;
use std::ops::Deref;
use std::sync::Arc;

impl<T: TLB> TLB for Arc<T> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TLCoreError> { Ok(Arc::new(T::read(parser)?)) }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TLCoreError> { self.deref().write(builder) }
}

impl<T: TLB> TLB for Box<T> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TLCoreError> { Ok(Box::new(T::read(parser)?)) }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TLCoreError> { self.deref().write(builder) }
}
