use crate::tests::test_tl_client::make_tonlib_client;
use ton_lib::contracts::contract_client::contract_client_cache::ContractClientCacheConfig;

#[tokio::test]
async fn test_contract_client_tl_data_provider() -> anyhow::Result<()> {
    let tl_client = make_tonlib_client(true, true).await?;
    let cache_config = ContractClientCacheConfig::default();
    // todo!();
    // let data_provider =
    // let mc_info = tl_client.get_mc_info().await?;
    // assert_ne!(mc_info.last.seqno, 0);

    Ok(())
}
