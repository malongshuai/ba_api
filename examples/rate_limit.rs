#![allow(dead_code, unused_variables)]

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
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 创建http连接
    let rest_conn = RestConn::new(api_key().unwrap(), sec_key().unwrap(), None).await;

    let b = rest_conn.dust_list().await;
    println!("{:?}", b)

    // 获取BTCUSDT的最近5根K线
    // let x = rest_conn
    //     .klines("BTCUSDT", "1m", None, None, Some(5u16))
    //     .await;
    // tracing::info!("{:?}", x);
}

/// 循环68次获取深度信息，每次获取深度消耗100权重值
#[allow(dead_code)]
async fn f(rest_conn: RestConn) {
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
        let t = tokio::spawn(async move {
            for sym in syms {
                // 该Rest请求消耗100权重值
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

/// 下单并撤单
#[allow(dead_code)]
async fn order(rest_conn: RestConn) {
    let res = rest_conn
        .limit_order("BTCUSDT", "buy", 20.0, 26000.0, None)
        .await
        .unwrap();
    println!("{:?}", res);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let res = rest_conn
        .cancel_order("BTCUSDT", Some(res.order_id()), None, None)
        .await;
    println!("{:?}", res);
}
