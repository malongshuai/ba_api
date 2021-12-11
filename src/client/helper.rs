use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn signature(key: &str, obj: &str) -> String {
    let key_bytes = key.as_bytes();
    let obj_bytes = obj.as_bytes();

    let sign_key = hmac::Key::new(hmac::HMAC_SHA256, key_bytes);
    let sign = hmac::sign(&sign_key, obj_bytes);
    hex::encode(sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature() {
        let key = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let obj = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        assert_eq!(
            signature(key, obj),
            "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71".to_string()
        )
    }
}
