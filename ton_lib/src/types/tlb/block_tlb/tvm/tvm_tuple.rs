use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::tvm::{VMCell, VMCellSlice, VMInt, VMStackValue, VMTinyInt};
use crate::types::tlb::tlb_type::{TLBPrefix, TLBType};
use num_bigint::BigInt;
use std::ops::{Deref, DerefMut};

macro_rules! extract_tuple_val {
    ($maybe_result:expr, $variant:ident) => {
        match &$maybe_result {
            None => Err(TonlibError::TVMStackEmpty),
            Some(VMStackValue::$variant(val)) => Ok(&val.value),
            Some(rest) => Err(TonlibError::TVMStackWrongType(stringify!($variant).to_string(), format!("{rest:?}"))),
        }
    };
}

// https://github.com/ton-blockchain/ton/blob/master/crypto/block/block.tlb#L872C30-L872C40
// Doesn't implement tlb schema directly for convenience purposes
// Very similar with VMStackValue, but random access to underlying values
#[derive(Debug, Clone, Default)]
pub struct VMTuple(Vec<VMStackValue>);

impl Deref for VMTuple {
    type Target = Vec<VMStackValue>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for VMTuple {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl VMTuple {
    pub fn new(items: Vec<VMStackValue>) -> Self { Self(items) }

    pub fn push_tiny_int(&mut self, value: i64) { self.push(VMStackValue::TinyInt(VMTinyInt { value })); }
    pub fn push_int(&mut self, value: BigInt) { self.push(VMStackValue::Int(VMInt { value })); }
    pub fn push_cell(&mut self, value: TonCellRef) { self.push(VMStackValue::Cell(VMCell { value })); }
    pub fn push_cell_slice(&mut self, cell: TonCellRef) {
        self.push(VMStackValue::CellSlice(VMCellSlice::from_cell(cell)));
    }

    pub fn get_tiny_int(&mut self, index: usize) -> Result<&i64, TonlibError> {
        extract_tuple_val!(self.get(index), TinyInt)
    }
    pub fn get_int(&mut self, index: usize) -> Result<&BigInt, TonlibError> { extract_tuple_val!(self.get(index), Int) }
    pub fn get_cell(&mut self, index: usize) -> Result<&TonCellRef, TonlibError> {
        extract_tuple_val!(self.get(index), Cell)
    }
    pub fn pop_cell_slice(&mut self, index: usize) -> Result<&TonCellRef, TonlibError> {
        extract_tuple_val!(self.get(index), CellSlice)
    }
}

impl TLBType for VMTuple {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x07, 8);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        let len: usize = parser.read_num(16)?;
        if len == 0 {
            return Ok(VMTuple(Vec::new()));
        }
        let mut data = Vec::with_capacity(len);
        read_tuple(parser, &mut data, len)?;
        Ok(VMTuple(data))
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> {
        builder.write_num(&self.len(), 16)?;
        if self.is_empty() {
            return Ok(());
        }
        write_tuple(builder, self, self.len())?;
        Ok(())
    }
}

fn read_tuple(parser: &mut CellParser, data: &mut Vec<VMStackValue>, rest_len: usize) -> Result<(), TonlibError> {
    read_tuple_ref(parser, data, rest_len - 1)?;
    data.push(VMStackValue::from_cell(parser.read_next_ref()?)?);
    Ok(())
}

fn read_tuple_ref(parser: &mut CellParser, data: &mut Vec<VMStackValue>, rest_len: usize) -> Result<(), TonlibError> {
    match rest_len {
        0 => {}
        1 => data.push(VMStackValue::from_cell(parser.read_next_ref()?)?),
        _ => {
            let mut ref_parser = parser.read_next_ref()?.parser();
            read_tuple(&mut ref_parser, data, rest_len)?
        }
    }
    Ok(())
}

fn write_tuple(builder: &mut CellBuilder, data: &[VMStackValue], rest_len: usize) -> Result<(), TonlibError> {
    write_tuple_ref(builder, data, rest_len - 1)?;
    builder.write_ref(data[rest_len - 1].to_cell_ref()?)
}

fn write_tuple_ref(builder: &mut CellBuilder, data: &[VMStackValue], rest_len: usize) -> Result<(), TonlibError> {
    match rest_len {
        0 => {}
        1 => builder.write_ref(data[0].to_cell_ref()?)?,
        _ => {
            let mut ref_builder = TonCell::builder();
            write_tuple(&mut ref_builder, data, rest_len)?;
            builder.write_ref(ref_builder.build()?.into_ref())?;
        }
    }
    Ok(())
}
