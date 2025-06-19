// use crate::tests::test_tl_client::make_tonlib_client;
// use std::str::FromStr;
// use tokio_test::assert_ok;
// use ton_lib::cell::ton_hash::TonHash;
// use ton_lib::clients::tonlibjson::TLClient;
// use ton_lib::contracts::jetton_master::JettonMaster;
// use ton_lib::contracts::jetton_wallet::JettonWallet;
// use ton_lib::contracts::methods::get_jetton_data::GetJettonData;
// use ton_lib::contracts::methods::get_wallet_address::GetWalletAddress;
// use ton_lib::contracts::methods::get_wallet_data::GetWalletData;
// use ton_lib::contracts::ton_contract::TonContractTrait;
// use ton_lib::contracts::wallet::WalletContract;
// use ton_lib::sys_utils::{sys_tonlib_client_set_verbosity_level, sys_tonlib_set_verbosity_level};
// use ton_lib::types::ton_address::TonAddress;
// use crate::tests::utils::{get_net_conf, init_logging};
//
// #[tokio::test]
// async fn test_contracts_all() -> anyhow::Result<()> {
//     let tl_client = make_tonlib_client(true, false).await?;
//     assert_jetton_wallet(&tl_client).await?;
//     assert_jetton_master(&tl_client).await?;
//     assert_wallet_contract(&tl_client).await?;
//     Ok(())
// }
//
// async fn assert_jetton_wallet(tl_client: &TLClient) -> anyhow::Result<()> {
//     let usdt_wallet = TonAddress::from_str("EQAmJs8wtwK93thF78iD76RQKf9Z3v2sxM57iwpZZtdQAiVM")?;
//     let contract = JettonWallet::new(usdt_wallet, tl_client.clone(), None).await?;
//     assert_ok!(contract.get_wallet_data().await);
//     Ok(())
// }
//
// async fn assert_jetton_master(tl_client: &TLClient) -> anyhow::Result<()> {
//     let usdt_master = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;
//     let contract = JettonMaster::new(usdt_master, tl_client.clone(), None).await?;
//     assert_ok!(contract.get_jetton_data().await);
//     let owner = TonAddress::from_str("UQAj-peZGPH-cC25EAv4Q-h8cBXszTmkch6ba6wXC8BM40qt")?;
//     let wallet = assert_ok!(contract.get_wallet_address(&owner).await);
//     assert_eq!(wallet.to_string(), "EQAmJs8wtwK93thF78iD76RQKf9Z3v2sxM57iwpZZtdQAiVM");
//     Ok(())
// }
//
// async fn assert_wallet_contract(tl_client: &TLClient) -> anyhow::Result<()> {
//     let wallet = TonAddress::from_str("UQAj-peZGPH-cC25EAv4Q-h8cBXszTmkch6ba6wXC8BM40qt")?;
//     let contract = WalletContract::new(wallet, tl_client.clone(), None).await?;
//     let seqno = contract.seqno().await?;
//     assert!(seqno > 0);
//     let public_key = contract.get_public_key().await?;
//     assert_ne!(public_key, TonHash::ZERO);
//     Ok(())
// }
//
// pub async fn make_contract_client(mainnet: bool, archive_only: bool) -> anyhow::Result<TLClient> {
//     init_logging();
//     log::info!("initializing tl_client with mainnet={mainnet}...");
//     let net_conf = get_net_conf(mainnet)?;
//     let config_types = ton_lib::clients::tonlibjson::TLClientConfig::new(net_conf, archive_only);
//     let tl_client = TLClient::new(config_types).await?;
//     sys_tonlib_set_verbosity_level(0);
//     sys_tonlib_client_set_verbosity_level(0);
//
//     let contract_cliie
//     Ok(tl_client)
// }
