use crate::get_chk_from_mpq_in_memory;
use sha2::Digest;

fn hash(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();

    hasher.update(bytes);

    format!("{:x}", hasher.finalize())
}

async fn get_mpq_extract_chk_hash(id: &str) -> String {
    let url = format!("https://scmscx.com/api/maps/{}", id);
    let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let chk = get_chk_from_mpq_in_memory(&bytes[..]).unwrap();
    hash(&chk)
}

#[tokio::test]
async fn can_extract_chks() {
    #[rustfmt::skip]
    const MPQS: &[(&str, &str)] = &[
        ("2d2da06aefad28ac7609948fce16838c1cea71bb38ba28f88deabbff08fa3e4f", "ea537b0ce9ed0dfdd0c3c027e8cb10f47532734d1e54c8b767185348c0eb8451"),
        ("31ebe03f56b224b7af28bdca735f7b976660b0f276d3e16b0308753d1869c610", "541bb96fdd38d14a6dc2cd877fa80d480e2e52ccd96e76e17e276deac4d23f52"),
        ("006c28caf8b5f47e1062ca77b89063160c5ba8d85ee681f3aaba5c5f4b6fadfb", "14d34632b03f2c929fd4e63349f69755d14cfddd29af910ba3adfef45f37730f"),
        ("18f3e26682dfdfc42113f5a6a924dae0c4eb50d3178dfa112ce922681554c384", "e1020e7169d92ffba63b44fa52f52ce8dd4281aaa0a27767518280ceb7b13d50"),
        ("8b1f43c8a071a3505b28afd641e27631f9248cae26315f12b20cfecab69f6ca8", "d12f9050eb0029756b763b2eb30a442acea627606d275714dca86664e15e3c92"),
        ("69d2a122e87d5ad823c7cf51f5cf1eca01f19fbc0f88e44bcd9fa5779d31e652", "edbeb33118923ced0f5305b5bca1d5bf5e94427f44a2d7cbf3b40d26840ed2f5"),
        ("80b9861d26fe36b4f81b79bb7faafeb1794e9d0f9adc27398a42954bc93713df", "bb2d1dd37853f60d35b372b4280e80956407aa1be21c6fd89275a18a0edc15f5"),
    ];

    for (mpq_hash, chk_hash) in MPQS {
        assert_eq!(get_mpq_extract_chk_hash(mpq_hash).await.as_str(), *chk_hash);
    }
}
