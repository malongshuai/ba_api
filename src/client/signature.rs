use ba_ed25519::decrypt_key;
use base64::{engine::general_purpose, Engine};
use ed25519_dalek::ed25519::signature::SignerMut as _;
use ring::hmac;
use serde::Serialize;

//@ 生成ed25519的公私钥：
//@ 生成私钥：openssl genpkey -algorithm ED25519 -out private_key.pem
//@ 生成公钥：openssl pkey -in private_key.pem -pubout -out public_key.pem

#[derive(Serialize)]
pub(crate) struct SignatureParam {
    signature: String,
}

/// 传递ed25519的私钥，可能是加密的可能是未加密的，都行，
/// 然后对obj payload进行签名，并将签名后的数据进行base64编码
pub(crate) fn ed25519_signature(key: &str, obj: &str) -> SignatureParam {
    let mut private_key = decrypt_key(key).unwrap();
    let sig = general_purpose::STANDARD.encode(private_key.sign(obj.as_bytes()).to_bytes());
    SignatureParam { signature: sig }
}

/// 传递secret key和要签名的payload
#[allow(dead_code)]
pub(crate) fn hmac_signature(key: &str, obj: &str) -> SignatureParam {
    let key_bytes = key.as_bytes();
    let obj_bytes = obj.as_bytes();

    let sign_key = hmac::Key::new(hmac::HMAC_SHA256, key_bytes);
    let sign = hmac::sign(&sign_key, obj_bytes);
    SignatureParam {
        signature: hex::encode(sign),
    }
}

#[cfg(test)]
mod helper {
    use crate::client::signature::hmac_signature;

    #[test]
    fn test_signature() {
        let key = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let obj = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        assert_eq!(
            hmac_signature(key, obj).signature,
            "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71".to_string()
        )
    }
}
