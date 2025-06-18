use crate::tests::test_tl_client::make_tonlib_client;
use std::collections::{HashMap, HashSet};
use ton_lib::contracts::contract_client::block_stream::BlockStream;
use ton_lib::contracts::contract_client::contract_client_cache::ContractClientCacheConfig;

#[tokio::test]
async fn test_contract_client_tl_data_provider() -> anyhow::Result<()> {
    let cache_config = ContractClientCacheConfig::default();
    // todo!();
    // let data_provider =
    // let mc_info = tl_client.get_mc_info().await?;
    // assert_ne!(mc_info.last.seqno, 0);

    Ok(())
}

#[tokio::test]
async fn test_block_stream() -> anyhow::Result<()> {
    let tl_client = make_tonlib_client(true, true).await?;
    // we run it from 230 to fill prev_ids properly, but test only cover 234 & 235
    let from_seqno = 3_800_234;
    let to_seqno = 3_800_235;
    let mut block_stream = BlockStream::new(tl_client, from_seqno, Some(to_seqno)).await?;

    let expected_shards = HashMap::from([
        (
            3_800_234,
            HashSet::from([
                (-1, -9223372036854775808i64, 3800234),
                (0, -8646911284551352320, 5254686),
                (0, -8646911284551352320, 5254687),
                (0, -7493989779944505344, 5255355),
                (0, -6341068275337658368, 5254128),
                (0, -6341068275337658368, 5254129),
                (0, -5188146770730811392, 5254058),
                (0, -4035225266123964416, 5254419),
                (0, -2882303761517117440, 5252486),
                (0, -1729382256910270464, 5254503),
                (0, -1729382256910270464, 5254504),
                (0, -576460752303423488, 5252885),
                (0, -576460752303423488, 5252886),
                (0, 576460752303423488, 5257367),
                (0, 576460752303423488, 5257368),
                (0, 1729382256910270464, 5254309),
                (0, 2882303761517117440, 5253219),
                (0, 4035225266123964416, 5255503),
                (0, 5188146770730811392, 5250846),
                (0, 6341068275337658368, 5252412),
                (0, 6341068275337658368, 5252413),
                (0, 7493989779944505344, 5254150),
                (0, 7493989779944505344, 5254151),
                (0, 8646911284551352320, 5253528),
                (0, 8646911284551352320, 5253529),
            ]),
        ),
        (3_800_235, HashSet::from([(-1, -9223372036854775808, 3800235), (0, 7493989779944505344, 5254152)])),
    ]);

    let given_vec = block_stream.next().await?.unwrap();
    assert_eq!(given_vec.last().unwrap().shard_id.wc, -1);
    let given_set = given_vec.into_iter().map(|x| (x.shard_id.wc, x.shard_id.shard as i64, x.seqno)).collect();
    let expected_set = &expected_shards[&3_800_234];
    assert_eq!(expected_set, &given_set);

    let given_vec = block_stream.next().await?.unwrap();
    assert_eq!(given_vec.last().unwrap().shard_id.wc, -1);
    let given_set = given_vec.into_iter().map(|x| (x.shard_id.wc, x.shard_id.shard as i64, x.seqno)).collect();
    let expected_set = &expected_shards[&3_800_235];
    assert_eq!(expected_set, &given_set);

    assert!(block_stream.next().await?.is_none());
    Ok(())
}
