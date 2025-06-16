use crate::cell::ton_cell::TonCellRef;
use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterInto;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterTLB;
use crate::types::tlb::adapters::tlb_hash_map::TLBHashMap;
use crate::types::tlb::adapters::ConstLen;
use crate::types::tlb::adapters::TLBRef;
use crate::types::tlb::block_tlb::tvm::tvm_cell_slice::TVMCellSlice;
use crate::types::tlb::block_tlb::tvm::tvm_stack::TVMStack;
use crate::types::tlb::block_tlb::tvm::tvm_tuple::TVMTuple;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use ton_lib_macros::TLBDerive;

#[derive(Clone, TLBDerive)]
pub enum TVMStackValue {
    Null(TVMNull),
    TinyInt(TVMTinyInt),
    Int(TVMInt),
    Nan(TVMNan),
    Cell(TVMCell),
    CellSlice(TVMCellSlice),
    Builder(TVMBuilder), // TODO is not tested
    Cont(TVMCont),       // TODO is not tested
    Tuple(TVMTuple),
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x00, bits_len = 8)]
pub struct TVMNull;

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x01, bits_len = 8)]
pub struct TVMTinyInt {
    pub value: i64,
}

// vm_stk_int#0201_ value:int257 = VmStackValue; means 0x0201 without latest bit ==> 0000001000000000
#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x0100, bits_len = 15)]
pub struct TVMInt {
    #[tlb_derive(bits_len = 257)]
    pub value: BigInt,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x02ff, bits_len = 16)]
pub struct TVMNan;

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x03, bits_len = 8)]
pub struct TVMCell {
    pub value: TonCellRef,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x05, bits_len = 8)]
pub struct TVMBuilder {
    pub cell: TonCellRef,
}

#[derive(Debug, Clone, TLBDerive)]
pub enum TVMCont {
    Std(VMContStd),
    Envelope(TVMContEnvelope),
    Quit(VMContQuit),
    QuitExc(TVMContQuitExc),
    Repeat(VMContRepeat),
    Until(VMContUntil),
    Again(VMContAgain),
    WhileCond(VMContWhileCond),
    WhileBody(VMContWhileBody),
    PushInt(VMContPushInt),
}

#[derive(Debug, Clone, TLBDerive)]
pub struct VMControlData {
    #[tlb_derive(bits_len = 13)]
    pub nargs: Option<u16>,
    pub stack: Option<Arc<TVMStack>>,
    pub save: VMSaveList,
    pub cp: Option<i16>,
}

#[derive(Debug, Clone, TLBDerive)]
pub struct VMSaveList {
    #[tlb_derive(adapter = "TLBHashMap::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(4)")]
    pub cregs: HashMap<u8, TVMStackValue>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x00, bits_len = 8)]
pub struct VMContStd {
    pub data: Arc<VMControlData>,
    pub code: Arc<TVMCellSlice>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x01, bits_len = 8)]
pub struct TVMContEnvelope {
    pub data: VMControlData,
    #[tlb_derive(adapter = "TLBRef")]
    pub next: Arc<TVMCont>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x1000, bits_len = 16)]
pub struct VMContQuit {
    pub exit_code: i32,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x1001, bits_len = 16)]
pub struct TVMContQuitExc {}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x10100, bits_len = 20)]
pub struct VMContRepeat {
    #[tlb_derive(bits_len = 63)]
    pub count: u64,
    #[tlb_derive(adapter = "TLBRef")]
    pub body: Arc<TVMCont>,
    #[tlb_derive(adapter = "TLBRef")]
    pub after: Arc<TVMCont>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x110000, bits_len = 24)]
pub struct VMContUntil {
    #[tlb_derive(adapter = "TLBRef")]
    pub body: Arc<TVMCont>,
    #[tlb_derive(adapter = "TLBRef")]
    pub after: Arc<TVMCont>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x110001, bits_len = 24)]
pub struct VMContAgain {
    #[tlb_derive(adapter = "TLBRef")]
    pub body: Arc<TVMCont>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x110010, bits_len = 24)]
pub struct VMContWhileCond {
    #[tlb_derive(adapter = "TLBRef")]
    pub cond: Arc<TVMCont>,
    #[tlb_derive(adapter = "TLBRef")]
    pub body: Arc<TVMCont>,
    #[tlb_derive(adapter = "TLBRef")]
    pub after: Arc<TVMCont>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x110011, bits_len = 24)]
pub struct VMContWhileBody {
    #[tlb_derive(adapter = "TLBRef")]
    pub cond: Arc<TVMCont>,
    #[tlb_derive(adapter = "TLBRef")]
    pub body: Arc<TVMCont>,
    #[tlb_derive(adapter = "TLBRef")]
    pub after: Arc<TVMCont>,
}

#[derive(Debug, Clone, TLBDerive)]
#[tlb_derive(prefix = 0x1111, bits_len = 16)]
pub struct VMContPushInt {
    pub value: i32,
    #[tlb_derive(adapter = "TLBRef")]
    pub next: Arc<TVMCont>,
}

impl Debug for TVMStackValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") }
}

impl Display for TVMStackValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TVMStackValue::Null(_) => write!(f, "Null"),
            TVMStackValue::TinyInt(v) => write!(f, "TinyInt({})", v.value),
            TVMStackValue::Int(v) => write!(f, "Int({})", v.value),
            TVMStackValue::Nan(_) => write!(f, "Nan"),
            TVMStackValue::Cell(v) => write!(f, "Cell({})", v.value.deref()),
            TVMStackValue::CellSlice(v) => write!(f, "CellSlice({})", v.value.deref()),
            TVMStackValue::Builder(_) => write!(f, "Builder"),
            TVMStackValue::Cont(_) => write!(f, "Cont"),
            TVMStackValue::Tuple(v) => write!(f, "Tuple[{v:?}"),
        }
    }
}
