use crate::tests::utils::make_tl_client;
use std::str::FromStr;
use std::time::Duration;
use tokio_test::assert_ok;
use ton_lib::clients::tl_client::tl::client::TLClientTrait;
use ton_lib::contracts::contract_client::ContractClient;
use ton_lib::contracts::jetton_master::JettonMaster;
use ton_lib::contracts::methods::get_jetton_data::GetJettonData;
use ton_lib::contracts::methods::get_wallet_address::GetWalletAddress;
use ton_lib::contracts::tl_provider::provider::TLProvider;
use ton_lib::contracts::tl_provider::provider_config::TLProviderConfig;
use ton_lib::contracts::ton_contract::TonContract;
use ton_lib_core::types::TonAddress;

#[tokio::test]
async fn test_tl_provider() -> anyhow::Result<()> {
    let tl_client = make_tl_client(true, true).await?;

    let head_seqno = tl_client.get_mc_info().await?.last.seqno;

    let provider_config = TLProviderConfig::new_no_cache(head_seqno);
    let tl_provider = TLProvider::new(provider_config, tl_client.clone()).await?;
    let _ctr_cli = ContractClient::new(tl_provider)?;
    Ok(())
}

#[tokio::test]
async fn test_emulate_get_method_cache() -> anyhow::Result<()> {
    let tl_client = make_tl_client(true, true).await?;
    let head_seqno = tl_client.get_mc_info().await?.last.seqno;
    let provider_config = TLProviderConfig::new(head_seqno, 1000, Duration::from_secs(60));
    let data_provider = TLProvider::new(provider_config, tl_client.clone()).await?;
    let ctr_cli = ContractClient::new(data_provider)?;

    assert_eq!(ctr_cli.get_cache_stats().await?.get("emulate_get_method_req").copied(), Some(0));
    assert_eq!(ctr_cli.get_cache_stats().await?.get("emulate_get_method_miss").copied(), Some(0));

    let usdt_master = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;
    let contract = JettonMaster::new(&ctr_cli, usdt_master, None)?;
    let owner = TonAddress::from_str("UQAj-peZGPH-cC25EAv4Q-h8cBXszTmkch6ba6wXC8BM40qt")?;

    assert_ok!(contract.get_jetton_data().await);
    assert_ok!(contract.get_jetton_data().await);
    assert_ok!(contract.get_jetton_data().await);

    assert_ok!(contract.get_wallet_address(&owner).await);
    assert_ok!(contract.get_wallet_address(&owner).await);
    assert_ok!(contract.get_wallet_address(&owner).await);

    assert_eq!(ctr_cli.get_cache_stats().await?.get("emulate_get_method_req").copied(), Some(6));
    assert_eq!(ctr_cli.get_cache_stats().await?.get("emulate_get_method_miss").copied(), Some(2));

    Ok(())
}
