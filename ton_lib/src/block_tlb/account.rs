use crate::block_tlb::{Coins, CurrencyCollection, StateInit};
use crate::tlb_adapters::TLBRef;
use ton_lib_core::cell::{TonCellRef, TonHash};
use ton_lib_core::types::tlb_core::{MsgAddressInt, VarLenBytes};
use ton_lib_core::TLBDerive;

#[derive(Default, Debug, Clone, PartialEq, TLBDerive)]
pub struct ShardAccount {
    #[tlb_derive(adapter = "TLBRef")]
    pub account: MaybeAccount,
    pub last_tx_hash: TonHash,
    pub last_tx_lt: u64,
}

// https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L259
// intentionally implemented as enum - Account can't be used directly
#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum MaybeAccount {
    None(AccountNone),
    #[rustfmt::skip]
    Account(Box::<Account>),
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b0, bits_len = 1)]
pub struct AccountNone;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b1, bits_len = 1)]
pub struct Account {
    pub addr: MsgAddressInt,
    pub storage_stat: StorageInfo,
    pub storage: AccountStorage,
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub struct StorageUsed {
    pub cells: VarLenBytes<u64, 3>,
    pub bits: VarLenBytes<u64, 3>,
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub struct StorageInfo {
    pub used: StorageUsed,
    pub storage_extra: MaybeStorageExtraInfo,
    pub last_paid: u32,
    pub due_payment: Option<Coins>,
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub struct AccountStorage {
    pub last_tx_lt: u64,
    pub balance: CurrencyCollection,
    pub state: AccountState,
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum MaybeStorageExtraInfo {
    None(StorageExtraInfoNone),
    Info(StorageExtraInfo),
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b000, bits_len = 3)]
pub struct StorageExtraInfoNone;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b001, bits_len = 3)]
pub struct StorageExtraInfo {
    pub dict_hash: TonHash,
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum AccountState {
    Uninit(AccountStateUninit),
    Frozen(AccountStateFrozen),
    Active(AccountStateActive),
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b00, bits_len = 2)]
pub struct AccountStateUninit;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b01, bits_len = 2)]
pub struct AccountStateFrozen {
    pub state_hash: TonHash,
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b1, bits_len = 1)]
pub struct AccountStateActive {
    pub state_init: StateInit,
}

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L271
#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum AccountStatus {
    Uninit(AccountStatusUninit),
    Frozen(AccountStatusFrozen),
    Active(AccountStatusActive),
    NonExist(AccountStatusNotExist),
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b00, bits_len = 2)]
pub struct AccountStatusUninit;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b01, bits_len = 2)]
pub struct AccountStatusFrozen;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b10, bits_len = 2)]
pub struct AccountStatusActive;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b11, bits_len = 2)]
pub struct AccountStatusNotExist;

impl ShardAccount {
    pub const NON_EXIST: ShardAccount = ShardAccount {
        account: MaybeAccount::None(AccountNone),
        last_tx_hash: TonHash::ZERO,
        last_tx_lt: 0,
    };
}

impl Default for AccountStatus {
    fn default() -> Self { AccountStatus::NonExist(AccountStatusNotExist) }
}

impl Default for MaybeAccount {
    fn default() -> Self { MaybeAccount::None(AccountNone) }
}

#[rustfmt::skip]
impl MaybeAccount {
    pub fn as_active(&self) -> Option<&AccountStateActive> { self.as_account()?.storage.state.as_active() }
    pub fn as_frozen(&self) -> Option<&AccountStateFrozen> { self.as_account()?.storage.state.as_frozen() }
    pub fn get_code(&self) -> Option<&TonCellRef> { self.as_active()?.state_init.code.as_ref() }
    pub fn get_data(&self) -> Option<&TonCellRef> { self.as_active()?.state_init.data.as_ref() }
    pub fn get_balance(&self) -> Option<&Coins> { Some(&self.as_account()?.storage.balance.grams) }

    pub fn as_active_mut(&mut self) -> Option<&mut AccountStateActive> { self.as_account_mut()?.storage.state.as_active_mut() }
    pub fn as_frozen_mut(&mut self) -> Option<&mut AccountStateFrozen> { self.as_account_mut()?.storage.state.as_frozen_mut() }
    pub fn get_code_mut(&mut self) -> Option<&mut TonCellRef> { self.as_active_mut()?.state_init.code.as_mut() }
    pub fn get_data_mut(&mut self) -> Option<&mut TonCellRef> { self.as_active_mut()?.state_init.data.as_mut() }
    pub fn get_balance_mut(&mut self) -> Option<&mut Coins> { Some(&mut self.as_account_mut()?.storage.balance.grams) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_tlb::{SimpleLib, TickTock};
    use std::collections::HashMap;
    use std::str::FromStr;
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::tlb_core::{MsgAddressIntStd, VarLen};

    #[test]
    fn test_block_tlb_account_some() -> anyhow::Result<()> {
        let account_boc = "b5ee9c7201021d0100066d000271c00b113a994b5024a16719f69139328eb759596c38a25f59028b146fecdc3621dfe23a8bce83401229200000cc73d58b950d75499e8106934001020114ff00f4a413f4bcf2c80b030253705148e3baabcb0800c881fc78d28207072c728a2e7896228f37e17369ae121cb0eef7b4b0385f3330401a1b02016204050202cb0607020120161702f3d0cb434c0c05c6c238ecc200835c874c7c0608405e351466ea44c38601035c87e800c3b51343e803e903e90353534541168504d3214017e809400f3c58073c5b333327b55383e903e900c7e800c7d007e800c7e80004c5c3e0e80b4c7c04074cfc044bb51343e803e903e9035353449a084190adf41eeb8c089a0809001da23864658380e78b64814183fa0bc0019635355161c705f2e04904fa4021fa4430c000f2e14dfa00d4d120d0d31f018210178d4519baf2e0488040d721fa00fa4031fa4031fa0020d70b009ad74bc00101c001b0f2b19130e254431b0a03fa82107bdd97deba8ee7363805fa00fa40f82854120a70546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c9f9007074c8cb02ca07cbffc9d05008c705f2e04a12a14414506603c85005fa025003cf1601cf16ccccc9ed54fa40d120d70b01c000b3915be30de02682102c76b973bae30235250c0d0e018e2191729171e2f839206e938124279120e2216e94318128739101e25023a813a0738103a370f83ca00270f83612a00170f836a07381040982100966018070f837a0bcf2b025597f0b00ec82103b9aca0070fb02f828450470546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c920f9007074c8cb02ca07cbffc9d0c8801801cb0501cf1658fa02029858775003cb6bcccc9730017158cb6acce2c98011fb005005a04314c85005fa025003cf1601cf16ccccc9ed540044c8801001cb0501cf1670fa027001cb6a8210d53276db01cb1f0101cb3fc98042fb0001fc145f04323401fa40d2000101d195c821cf16c9916de2c8801001cb055004cf1670fa027001cb6a8210d173540001cb1f500401cb3f23fa4430c0008e35f828440470546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c9f9007074c8cb02ca07cbffc9d012cf1697316c127001cb01e2f400c90f04f882106501f354ba8e223134365145c705f2e04902fa40d1103402c85005fa025003cf1601cf16ccccc9ed54e0258210fb88e119ba8e2132343603d15131c705f2e0498b025512c85005fa025003cf1601cf16ccccc9ed54e034248210235caf52bae30237238210cb862902bae302365b2082102508d66abae3026c311011121300088050fb0002ec3031325033c705f2e049fa40fa00d4d120d0d31f01018040d7212182100f8a7ea5ba8e4d36208210595f07bcba8e2c3004fa0031fa4031f401d120f839206e943081169fde718102f270f8380170f836a0811a7770f836a0bcf2b08e138210eed236d3ba9504d30331d19434f2c048e2e2e30d50037014150044335142c705f2e049c85003cf16c9134440c85005fa025003cf1601cf16ccccc9ed54001e3002c705f2e049d4d4d101ed54fb0400188210d372158cbadc840ff2f000ce31fa0031fa4031fa4031f401fa0020d70b009ad74bc00101c001b0f2b19130e25442162191729171e2f839206e938124279120e2216e94318128739101e25023a813a0738103a370f83ca00270f83612a00170f836a07381040982100966018070f837a0bcf2b000c082103b9aca0070fb02f828450470546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c920f9007074c8cb02ca07cbffc9d0c8801801cb0501cf1658fa02029858775003cb6bcccc9730017158cb6acce2c98011fb000025bd9adf6a2687d007d207d206a6a6888122f82402027118190085adbcf6a2687d007d207d206a6a688a2f827c1400b82a3002098a81e46581ac7d0100e78b00e78b6490e4658089fa00097a00658064fc80383a6465816503e5ffe4e84000cfaf16f6a2687d007d207d206a6a68bf99e836c1783872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e62c780a417877407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f4589b4780a38646583fa0064a180400842028f452d7a4dfd74066b682365177259ed05734435be76b5fd4bd5d8af2b7c3d6801001c003e68747470733a2f2f7465746865722e746f2f757364742d746f6e2e6a736f6e";
        let cell = TonCell::from_boc_hex(account_boc)?;
        let account = MaybeAccount::from_cell(&cell)?;
        if let MaybeAccount::Account(account) = &account {
            assert_eq!(
                account.addr,
                MsgAddressIntStd {
                    anycast: None,
                    workchain: 0,
                    address: TonHash::from_str("B113A994B5024A16719F69139328EB759596C38A25F59028B146FECDC3621DFE")?,
                }
                .into()
            );
            assert_eq!(account.storage_stat.used.cells, VarLenBytes::new(29u32, 8));
            assert_eq!(account.storage_stat.used.bits, VarLenBytes::new(12090u32, 16));

            assert_eq!(account.storage.last_tx_lt, 56199469000003u64);
            assert_eq!(account.storage.balance, CurrencyCollection::new(915473564698u64));
            if let AccountState::Active(state) = &account.storage.state {
                let code = state.state_init.code.as_ref().unwrap();
                assert_eq!(
                    code.hash(),
                    &TonHash::from_str("18d5b6e780ff0bb451254c2c760d09d6e485638cd1407abb97078752c3c1c9ee")?
                );
            }
        } else {
            panic!("Expected Some account");
        }
        let serialized_back = account.to_cell()?;
        assert_eq!(serialized_back, cell);
        Ok(())
    }

    #[test]
    fn test_block_tlb_shard_account_tick_tock() -> anyhow::Result<()> {
        let boc_hex = "b5ee9c7201020d0100017500015099602ce40fd84286bddb06f8bcc9fceb7e3027f9826c8985017f16cba12363cc000016e2cc89c18101036fcff34517c7bdf5187c55af4f8b61fdc321588c7ab768dee24b006df29106458d7cf21881f4800000000000005b8b322706090311d3e017f009080202016206030142bf412429205ea66d6f2004edfa570f6f56b3e85e59baa1befbc73b7da5d55bdc61040104123405000456780142bf5a2eef5056775f5b9572ff3ad63dd2a71d1fb281ca177a5e1c74730eccb2e51307000fabacabadabacaba8004811fd096c00000000000000000000000000000000000000000000000000000000000000000114ff00f4a413f4a0f2c80b0a0201200c0b00dfa5ffff76a268698fe9ffe8e42c5267858f90e785ffe4f6aa6467c444ffb365ffc10802faf0807d014035e7a064b87d804077e7857fc10803dfd2407d014035e7a064b86467cd8903a32b9ba4410803ade68afd014035e7a045ea432b6363796103bb7b9363210c678b64b87d807d80400002d2";

        let shard_account = ShardAccount::from_boc_hex(boc_hex)?;
        let expected = ShardAccount {
            account: MaybeAccount::Account(Box::new(Account{
                addr: MsgAddressInt::Std(MsgAddressIntStd {
                    anycast: None,
                    workchain: -1,
                    address: TonHash::from_str("34517c7bdf5187c55af4f8b61fdc321588c7ab768dee24b006df29106458d7cf")?,
                }),
                storage_stat: StorageInfo {
                    used: StorageUsed {
                        cells: VarLenBytes::new(12u32, 8),
                        bits: VarLenBytes::new(2002u32, 16),
                    },
                    storage_extra: StorageExtraInfoNone.into(),
                    last_paid: 0,
                    due_payment: None,
                },
                storage: AccountStorage {
                    last_tx_lt: 25163350000002,
                    balance: CurrencyCollection::new(206000000u32),
                    state: AccountState::Active(AccountStateActive {
                        state_init: StateInit {
                            split_depth: None,
                            tick_tock: Some(TickTock {
                                tick: true,
                                tock: true,
                            }),
                            code: Some(TonCellRef::from_boc_hex("b5ee9c72010104010087000114ff00f4a413f4a0f2c80b01020120030200dfa5ffff76a268698fe9ffe8e42c5267858f90e785ffe4f6aa6467c444ffb365ffc10802faf0807d014035e7a064b87d804077e7857fc10803dfd2407d014035e7a064b86467cd8903a32b9ba4410803ade68afd014035e7a045ea432b6363796103bb7b9363210c678b64b87d807d80400002d2")?),
                            data: Some(TonCellRef::from_boc_hex("b5ee9c7201010101002600004811fd096c0000000000000000000000000000000000000000000000000000000000000000")?),
                            library: HashMap::from([
                                (TonHash::from_str("0D1777A82B3BAFADCAB97F9D6B1EE9538E8FD940E50BBD2F0E3A398766597289")?, SimpleLib {
                                    public: true,
                                    root: TonCellRef::from_boc_hex("b5ee9c7201010101000a00000fabacabadabacaba8")?,
                                }),
                                (TonHash::from_str("209214902F5336B7900276FD2B87B7AB59F42F2CDD50DF7DE39DBED2EAADEE30")?, SimpleLib {
                                    public: true,
                                    root: TonCellRef::from_boc_hex("b5ee9c7201010201000900010412340100045678")?,
                                }),

                            ]),
                        },
                    }),
                },
            })),
            last_tx_hash: TonHash::from_str("99602ce40fd84286bddb06f8bcc9fceb7e3027f9826c8985017f16cba12363cc")?,
            last_tx_lt: 25163350000001,
        };

        assert_eq!(expected, shard_account);

        assert_eq!(
            shard_account.cell_hash()?,
            TonHash::from_str("2EF34B7D264FC0C21713BE018B9FBB264B0AF887FF5715C36229BDF79B11A858")?
        );
        let serialized = shard_account.to_boc_hex()?;
        let parsed_back = ShardAccount::from_boc_hex(&serialized)?;
        assert_eq!(parsed_back, shard_account);
        Ok(())
    }

    #[test]
    fn test_block_tlb_shard_account_regular() -> anyhow::Result<()> {
        // https://tonviewer.com/transaction/cd4c4f0f3e7962b90c92f5f0c27967fd4468acfa15d4df50faf8d2704a489e0b on height 44489966
        let boc_hex = "b5ee9c7201021401000421000150b78a4a3e91ae0ddf8c49983a554e010cc4764ccc990500728e8202f958c7fc40000030a3c2065f4501026fc00949a19cfd6eb82bb5ff6573b11208c71abb9398411b3b4672f78a7e34ea706d92268715433ce498700000c28f08197d210a9949ea1340030201957081353c31caacd80129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b001ece9afb55cc82c82739247aa35879be66afeb1502a81a72f2a982ec7625b5fb20030114ff00f4a413f4bcf2c80b040201620605001ba0f605da89a1f401f481f481a8610202cc11070201200b080201480a090083200835c87b51343e803e903e90350c0134c7e08405e3514654882ea0841ef765f784ee84ac7cb8b174cfcc7e800c04e81408f214013e809633c58073c5b3327b552000db3b51343e803e903e90350c01f4cffe803e900c145468549271c17cb8b049f0bffcb8b0a0823938702a8005a805af3cb8b0e0841ef765f7b232c7c572cfd400fe8088b3c58073c5b25c60063232c14933c59c3e80b2dab33260103ec01004f214013e809633c58073c5b3327b55200201580f0c01f53b51343e803e903e90350c0234cffe80145468017e903e9002fe911d3232c084b281f2fff27414d431c1551cdb48965c150804d50500f214013e809633c58073c5b33248a0079c7232c032c132c004bd003d0032c032407e910c6af8407e40006ab84061386c2c5c1d3232c0b281f2fff2741631c16c7cb8b0c2a00d01fefa0051a8a18208989680820898968012b608a18208e4e1c0a018a1278e385279a018a182107362d09cc8cb1f5230cb3f58fa025007cf165007cf16c9718010c8cb0524cf165006fa0215cb6a14ccc971fb00102410239710491038375f04e225d70b01c30023c200b093356c21e30d03c85004fa0258cf1601cf16ccc9ed540e00428210d53276db708010c8cb055008cf165004fa0216cb6a12cb1f12cb3fc972fb0001f300f4cffe803e90087c007b51343e803e903e90350c144da8548ab1c17cb8b04a30bffcb8b0951d009c150804d50500f214013e809633c58073c5b33248a0079c7232c032c132c004bd003d0032c0325481be910c6af8407e40006ab84061386c2c5c1d3232c0b281f2fff274013e903d010c7e800835d27080201000d8f2e2c4778018c8cb055008cf1670fa0217cb6b17cc8210178d4519c8cb1f19cb3f5007fa0222cf165006cf1624fa025003cf16c95005cc2291729171e25008a812a08208e4e1c0aa008208989680a0a014bcf2e2c504c98040fb004130c85004fa0258cf1601cf16ccc9ed540201d4131200113e910c1c2ebcb8536000c30831c02497c138007434c0c05c6c2544d7c0fc03783e903e900c7e800c5c75c87e800c7e800c1cea6d0000b4c7e08403e29fa954882ea54c4d167c02b8208405e3514654882ea58c511100fc02f80d60841657c1ef2ea4d67c033817c12103fcbc20";

        let shard_account = ShardAccount::from_boc_hex(boc_hex)?;
        let expected = ShardAccount {
            account: MaybeAccount::Account(Box::new(Account{
                addr: MsgAddressInt::Std(MsgAddressIntStd {
                    anycast: None,
                    workchain: 0,
                    address: TonHash::from_str("949a19cfd6eb82bb5ff6573b11208c71abb9398411b3b4672f78a7e34ea706d9")?,
                }),
                storage_stat: StorageInfo {
                    used: StorageUsed {
                        cells: VarLen::new(19u32, 8),
                        bits: VarLen::new(7253u32, 16),
                    },
                    storage_extra: StorageExtraInfoNone.into(),
                    last_paid: 1738314510,
                    due_payment: None,
                },
                storage: AccountStorage {
                    last_tx_lt: 53479893000008,
                    balance: CurrencyCollection::new(711272360u32),
                    state: AccountState::Active(AccountStateActive {
                        state_init: StateInit {
                            split_depth: None,
                            tick_tock: None,
                            code: Some(TonCellRef::from_boc_hex("b5ee9c720102110100036c000114ff00f4a413f4bcf2c80b010201620302001ba0f605da89a1f401f481f481a8610202cc0e04020120080502014807060083200835c87b51343e803e903e90350c0134c7e08405e3514654882ea0841ef765f784ee84ac7cb8b174cfcc7e800c04e81408f214013e809633c58073c5b3327b552000db3b51343e803e903e90350c01f4cffe803e900c145468549271c17cb8b049f0bffcb8b0a0823938702a8005a805af3cb8b0e0841ef765f7b232c7c572cfd400fe8088b3c58073c5b25c60063232c14933c59c3e80b2dab33260103ec01004f214013e809633c58073c5b3327b55200201580c0901f53b51343e803e903e90350c0234cffe80145468017e903e9002fe911d3232c084b281f2fff27414d431c1551cdb48965c150804d50500f214013e809633c58073c5b33248a0079c7232c032c132c004bd003d0032c032407e910c6af8407e40006ab84061386c2c5c1d3232c0b281f2fff2741631c16c7cb8b0c2a00a01fefa0051a8a18208989680820898968012b608a18208e4e1c0a018a1278e385279a018a182107362d09cc8cb1f5230cb3f58fa025007cf165007cf16c9718010c8cb0524cf165006fa0215cb6a14ccc971fb00102410239710491038375f04e225d70b01c30023c200b093356c21e30d03c85004fa0258cf1601cf16ccc9ed540b00428210d53276db708010c8cb055008cf165004fa0216cb6a12cb1f12cb3fc972fb0001f300f4cffe803e90087c007b51343e803e903e90350c144da8548ab1c17cb8b04a30bffcb8b0951d009c150804d50500f214013e809633c58073c5b33248a0079c7232c032c132c004bd003d0032c0325481be910c6af8407e40006ab84061386c2c5c1d3232c0b281f2fff274013e903d010c7e800835d27080200d00d8f2e2c4778018c8cb055008cf1670fa0217cb6b17cc8210178d4519c8cb1f19cb3f5007fa0222cf165006cf1624fa025003cf16c95005cc2291729171e25008a812a08208e4e1c0aa008208989680a0a014bcf2e2c504c98040fb004130c85004fa0258cf1601cf16ccc9ed540201d4100f00113e910c1c2ebcb8536000c30831c02497c138007434c0c05c6c2544d7c0fc03783e903e900c7e800c5c75c87e800c7e800c1cea6d0000b4c7e08403e29fa954882ea54c4d167c02b8208405e3514654882ea58c511100fc02f80d60841657c1ef2ea4d67c033817c12103fcbc20")?),
                            data: Some(TonCellRef::from_boc_hex("b5ee9c72010212010003ba0001957081353c31caacd80129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b001ece9afb55cc82c82739247aa35879be66afeb1502a81a72f2a982ec7625b5fb20010114ff00f4a413f4bcf2c80b020201620403001ba0f605da89a1f401f481f481a8610202cc0f05020120090602014808070083200835c87b51343e803e903e90350c0134c7e08405e3514654882ea0841ef765f784ee84ac7cb8b174cfcc7e800c04e81408f214013e809633c58073c5b3327b552000db3b51343e803e903e90350c01f4cffe803e900c145468549271c17cb8b049f0bffcb8b0a0823938702a8005a805af3cb8b0e0841ef765f7b232c7c572cfd400fe8088b3c58073c5b25c60063232c14933c59c3e80b2dab33260103ec01004f214013e809633c58073c5b3327b55200201580d0a01f53b51343e803e903e90350c0234cffe80145468017e903e9002fe911d3232c084b281f2fff27414d431c1551cdb48965c150804d50500f214013e809633c58073c5b33248a0079c7232c032c132c004bd003d0032c032407e910c6af8407e40006ab84061386c2c5c1d3232c0b281f2fff2741631c16c7cb8b0c2a00b01fefa0051a8a18208989680820898968012b608a18208e4e1c0a018a1278e385279a018a182107362d09cc8cb1f5230cb3f58fa025007cf165007cf16c9718010c8cb0524cf165006fa0215cb6a14ccc971fb00102410239710491038375f04e225d70b01c30023c200b093356c21e30d03c85004fa0258cf1601cf16ccc9ed540c00428210d53276db708010c8cb055008cf165004fa0216cb6a12cb1f12cb3fc972fb0001f300f4cffe803e90087c007b51343e803e903e90350c144da8548ab1c17cb8b04a30bffcb8b0951d009c150804d50500f214013e809633c58073c5b33248a0079c7232c032c132c004bd003d0032c0325481be910c6af8407e40006ab84061386c2c5c1d3232c0b281f2fff274013e903d010c7e800835d27080200e00d8f2e2c4778018c8cb055008cf1670fa0217cb6b17cc8210178d4519c8cb1f19cb3f5007fa0222cf165006cf1624fa025003cf16c95005cc2291729171e25008a812a08208e4e1c0aa008208989680a0a014bcf2e2c504c98040fb004130c85004fa0258cf1601cf16ccc9ed540201d4111000113e910c1c2ebcb8536000c30831c02497c138007434c0c05c6c2544d7c0fc03783e903e900c7e800c5c75c87e800c7e800c1cea6d0000b4c7e08403e29fa954882ea54c4d167c02b8208405e3514654882ea58c511100fc02f80d60841657c1ef2ea4d67c033817c12103fcbc20")?),
                            library: Default::default(),
                        },
                    }),
                },
            })),
            last_tx_hash: TonHash::from_str("b78a4a3e91ae0ddf8c49983a554e010cc4764ccc990500728e8202f958c7fc40")?,
            last_tx_lt: 53479893000005,
        };

        assert_eq!(expected, shard_account);
        assert_eq!(
            shard_account.cell_hash()?,
            TonHash::from_str("355BCC314569D5A3627E374F709464D3F9E0126CDB71DAB860DF18C6867C40D4")?
        );
        let serialized = shard_account.to_boc_hex()?;
        let parsed_back = ShardAccount::from_boc_hex(&serialized)?;
        assert_eq!(parsed_back, shard_account);
        Ok(())
    }

    #[test]
    fn test_block_tlb_account_storage_extra_info() -> anyhow::Result<()> {
        let account = MaybeAccount::from_boc_hex("b5ee9c7201023401000f140002b1c00f81e9c8ef2294f1c7515e02a621ccab3f05d9e5b254e307d743314d8ad8dc49a2689c4cca438d42d5d95ecf4d0fcb68cc51d0838c320f8b655fd5c7c7f6aeb014ca26905b416ff08000007ed2d42dc0154b2036c72d134001020114ff00f4a413f4bcf2c80b030386801d02ec7498d846f039cc605d8cb55199be8ffa5008728151498d6b929c3b196d2400931f7fc347c05333778502c862ced26ef1dbd61ae84c0ec489dddbba1c25765830313202016204050198d0eda2edfb20c700925f04e001d0d3030171b0925f04e0fa40fa4031fa0031f401fa0031fa00013170f83a02d31f0101d33f01125365a18055820085ce8209e1338070f83766b608a15166a106020120272804f2ed44d0fa4001f861fa4001f862fa4001f863d401d0fa0001f86cfa0001f86bfa0030f86dd401f86ed430d0d32f01f86ad32f01f866d32f01f867d32f01f868d32f01f869d30301f864f40430f86522821048e660b5bae3022282106c582059ba9136e30d2182100e50d313bae302343434228210cf6a5da4ba0708090a00b46c7181179470f8365cbcf2e064f84d59a1a0f86df845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5402fc03d31ffa00f404fa4055302170547000246ec0008e156c3101d0d300fa00fa0023963403fa0030039130e29134e255027f266ec000f84cf84b5007a816a05240b915b0927034de5320a022a027a02e81334a70f836a0a052c0be5240b0e30210365f0638c88210ae7df95b2402cb1fcb3fcb1f21cf16c9702602801080420b0c03ec3101fa40d4d4f828f84e52300270705003c85003cf168b02cf16ca00ccc921c8cb0113f40012f400cb00c97001f90074c8cb0212ca07cbffc9d05006c705f2e04701d7393078d721d3ff0131f8458307f40e6fa131f2e0c920d0f404f401fa40fa003020c300935b3234e30ddb3c5252a1c200f2e0640f101103fc8f7a3202d4f828f84e52300270705003c85003cf168b02cf16ca00ccc921c8cb0113f40012f400cb00c97001f90074c8cb0212ca07cbffc9d05003c705f2e04701fa40308210cf6a5da4587f830771800cc8cb03cb01cb0813cbff02957158cb61cc987058cb6101d0cf16e2c98010fb0020d70b01c000b3915be30de02219251a01fe3339393a3b5266a05008a05008a026a08014fb02f84d5004a0f86df845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed54216ec0009d01c8f400f84cfa02f84bfa02c992316de205c8cb1f2101cb3f0d006e226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb00f80f06c000938100cc928064e2f2f0102501925003cf165003fa0213f40001cf16c982106c582059017f830771800cc8cb03cb01cb0813cbff02957158cb61cc987058cb6101d0cf16e2c98010fb0021d70b01c000b3915be30ddb310e0076c88210d53276db5802cb1fcb3fc970018010810082226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb0004fe250441381523d70b01c0008e63aa00814d6970f836a0738103b582100966018070f837a0801c8127e08209e1338070f837a07020c88210d7b9c06e580802cb1fcb3f23cf165004fa0258cf1622fa0214cb00cb00c970c87001ca0012cccb00c9c87001cbff58cf1658fa02ccc9c8ccc9e30d218307f48e6fa56c12e30f01d01213141502f670207f8ef6238307f47c6fa5208ee702d430d0d3ff0130fa40fa00d430d0d2000101d4f40430206e8e2e3054232080108011226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb008e90c802d012cf162310355980108011db3ce2019213a09414a04313e2029132e201b3161701fcf845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5482100e50d31350047f830771800cc8cb03cb01cb0813cbff02957158cb61cc987058cb6101d0cf16e2c98010fb005043a101a1f84d5210be18003e306c2270c8c9c87f01ca00cccb00c9c87001cbff58cf1658fa02ccc9c8ccc90002a400043070000a028307f41600a28e4dc85801cb055005cf165003fa0254712323ed41ed43ed44ed45ed479f5bc85003cf17c913775003cb6bcccced67ed65ed64ed63ed61747fed11987601cb6bcc01cf17ed41edf101f2ffc901fb00db060008e6306c1200a6f2e0658010fb0201fa403020d70b01c000b38e3cc88210d53276db580302cb1fcb3fc970018010810082226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb00915be20076c88210ae7df95b580302cb1fcb3fc9700180108042226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb0004f6821023b05641bae3022282101f95f86cba8ed46c22f8435210c705f2e048f84dc300f2e0c8c88210f358b6d0580302cb1fcb3ff84dfa02c9f84d01801072226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb0070f86de031218210063199b7bae3022182103531465cba1b251c1d01fe32f8435220c705f2e048f823f849bef2e0ca02d3ff01f84802d32f0131f868f848f849bef2e0cbf847f866f823f867f847f84aa0f869f823f844f84aa8a1f8458307f4866fa5908e1c01d32f013122bb9bf84552108307f45b30f865def8458307f47c6fa5e85f03c8f84901cb2fc9d0f84552208307f416f865708014fb021e00a631f841c705f2e046fa0030f86bf845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5404f88e5331f841c705f2e046fa0030f86cf845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed54e02182105cec6be0bae302218210581879bcbae30221821060094a1bbae3022182106a4fbe34ba2021222301d8f845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed54c80101cbff0101cb2ff84801cb2ff84601cb2ff84701cb2ff84901cb2fc9821023b05641017f1f00c2830771800cc8cb03cb01cb0813cbff02957158cb61cc987058cb6101d0cf16e2c98010fb00c88210d53276db580302cb1fcb3fc970018010810082226eb32091719170e2c8500401cb055006cf165004fa02cb6a039358cc019130e201c901fb0000a631f841c705f2e046fa4030f863f845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5400b831f841c705f2e046fa4030f862f8428b02c705f2d050f845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5400a66c21f841c705f2e0468b02f862f845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5401f88e576c21f842c705f2e049f842f8618b02f862f845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed54e021821020faec53ba8e1631f841c705f2e046d421fb0401d0ed1eed53d430ed54e0212402be82107ee5a6d0ba8ebd31f841c705f2e046d430f86e6df86582107ee5a6d0c8c970830771800cc8cb03cb01cb0813cbff02957158cb61cc987058cb6101d0cf16e2c98010fb00e0218210e97250b7bae30230318210d53276dbbadc840ff2f02526008cf845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed5400a831f841c705f2e046d32f0131f86af845f844c8f84a01cb2ff84601cb2ff84701cb2ff84801cb2ff84901cb2fcb03f400c9f84ec8f84cfa02f84bfa02f84dfa02c9c8f841cf16f842cf16f843cf16ccccccc9ed54020158292a0201202c2d00d9b444bda89a1f48003f0c3f48003f0c5f48003f0c7a803a1f40003f0d9f40003f0d7f40061f0dba803f0dda861a1a65e03f0d5a65e03f0cda65e03f0cfa65e03f0d1a65e03f0d3a60603f0c9e80861f0cbf083f085f087f089f08bf08df08ff091f095f093f099f097f09bf09d001f9b4817da89a1f48003f0c3f48003f0c5f48003f0c7a803a1f40003f0d9f40003f0d7f40061f0dba803f0dda861a1a65e03f0d5a65e03f0cda65e03f0cfa65e03f0d1a65e03f0d3a60603f0c9e80861f0cae041f08b060fe90cdf4b211c3003a65e0262a6077928d844a2212261c5f08b060fe8f8df4bd02046be07f08d02b0014f847f848f849f84af84401ebb9685ed44d0fa4001f861fa4001f862fa4001f863d401d0fa0001f86cfa0001f86bfa0030f86dd401f86ed430d0d32f01f86ad32f01f866d32f01f867d32f01f868d32f01f869d30301f864f40430f865f828f84e120270705003c85003cf168b02cf16ca00ccc921c8cb0113f40012f400cb00c92082e01e9bb316ed44d0fa4001f861fa4001f862fa4001f863d401d0fa0001f86cfa0001f86bfa0030f86dd401f86ed430d0d32f01f86ad32f01f866d32f01f867d32f01f868d32f01f869d30301f864f40430f865f828f84e120270705003c85003cf168b02cf16ca00ccc921c8cb0113f40012f400cb00c982f00207001f90074c8cb0212ca07cbffc9d001001e7001f90074c8cb0212ca07cbffc9d00019398968030f4240501f58373808084202e46fa11560c837d5226a87e1bec85bb89bf960c0077c8e4354a747242cba1b05013d00000000001e0000682dfce50000682dfdea0000682dfdd50000682dfe083c33004fa0005301f8059bf55cfa01c7f8cd4bb128402936204591ae294aa5f6f6009a911300000d05bfc110")?;
        assert_eq!(
            account.as_account().unwrap().storage_stat.storage_extra.as_info().unwrap().dict_hash,
            TonHash::from_str("4871A85ABB2BD9E9A1F96D198A3A10718641F16CABFAB8F8FED5D6029944D20B")?
        );
        Ok(())
    }

    #[test]
    fn test_block_tlb_account_with_other_currencies() -> anyhow::Result<()> {
        let account = MaybeAccount::from_boc_hex("b5ee9c7201020c01000135000477cff000000000000000000000000000000000000000000000000000000000000000021881c9400000000000000470ddcc451211229a017f31012033c00908070102016205020142bf412429205ea66d6f2004edfa570f6f56b3e85e59baa1befbc73b7da5d55bdc60030104123404000456780142bf5a2eef5056775f5b9572ff3ad63dd2a71d1fb281ca177a5e1c74730eccb2e51306000fabacabadabacaba800480000000a2270f7e042f16b5d2f84eaaf74dd726fe0f3214fae0db98fc9811c928d7b7a7d0098ff0020dd2082014c97ba9730ed44d0d70b1fe0a4f260810200d71820d70b1fed44d0d31fd3ffd15112baf2a122f901541044f910f2a2f80001d31f31d307d4d101fb00a4c8cb1fcbffc9ed540201200b0a0015bfffffffbcbd1a94a200100015be000003bcb3670dc15550")?;
        assert!(!account.as_account().unwrap().storage.balance.other.is_empty());
        Ok(())
    }
}
