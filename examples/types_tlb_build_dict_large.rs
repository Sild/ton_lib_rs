use num_bigint::BigUint;
use std::collections::HashMap;
use ton_lib::types::tlb::adapters::dict_key_adapters::DictKeyAdapterInto;
use ton_lib::types::tlb::adapters::dict_val_adapters::DictValAdapterNum;
use ton_lib::types::tlb::adapters::tlb_hash_map::TLBHashMap;
use ton_lib::{ton_lib_macros::TLBDerive, types::tlb::TLB};
extern crate num_bigint;
extern crate ton_lib;

// const ITEMS_COUNT: usize = 40000000;
const ITEMS_COUNT: usize = 400000;

#[derive(TLBDerive)]
struct MyDict {
    #[tlb_derive(adapter = "TLBHashMap::<DictKeyAdapterInto, DictValAdapterNum<256>, _, _>::new(256)")]
    pub data: HashMap<usize, BigUint>,
}

fn main() -> anyhow::Result<()> {
    let mut data = HashMap::new();
    for i in 0..ITEMS_COUNT {
        data.insert(i, BigUint::from(i));
    }
    let cell = MyDict { data }.to_cell()?;

    println!("{}", cell.hash());
    Ok(())
}
