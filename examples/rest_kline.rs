use ba_api::client::RestConn;

fn api_key() -> Option<String> {
    std::env::var("BA_API_KEY").ok()
}

fn sec_key() -> Option<String> {
    std::env::var("BA_SEC_KEY").ok()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 创建http连接
    let rest_conn = RestConn::new(api_key().unwrap(), sec_key().unwrap(), None).await;

    // 获取BTCUSDT的最近5根K线
    // let x = rest_conn
    //     .klines("BTCUSDT", "1m", None, None, Some(5u16))
    //     .await;
    // debug!("{:?}", x);

    // 循环20次获取深度信息
    let syms = [
        "BTCUSDT",
        "ETHUSDT",
        "DOGEUSDT",
        "DOTUSDT",
        "EOSUSDT",
        "MATICUSDT",
        "SANDUSDT",
        "XRPUSDT",
        "LTCUSDT",
        "AXSUSDT",
        "BAKEUSDT",
        "OGUSDT",
        "OPUSDT",
        "APEUSDT",
        "ARBUSDT",
        "TRXUSDT",
        "CHZUSDT",
    ];

    let mut tasks = vec![];
    for _ in 0..4 {
        let conn = rest_conn.clone();
        let syms = syms.clone();
        let t = tokio::spawn(async move {
            for sym in syms {
                let res = conn.depth(sym, Some(1001)).await;
                println!("{}, {:?}", sym, res.map(|x| x.first_update_id));
            }
        });
        tasks.push(t);
    }

    for t in tasks {
        t.await.unwrap();
    }
}
