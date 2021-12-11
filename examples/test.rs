use ba_api::base::{FloatPrecision, Truncate};

// trait Truncate {
//     fn trunc_to(&self, temp: &str) -> f64;
// }

// impl Truncate for f64 {
//     fn trunc_to(&self, temp: &str) -> f64 {
//         let f = temp.parse::<f64>().unwrap();
//         let i = f.trunc();
//         let frac = f - i;
//         let precision = (frac.to_string().len() - 2) as i32;
//         (self * 10f64.powi(precision)).floor().trunc() / 10f64.powi(precision)
//     }
// }
use serde_yaml::{self, Value};
use std::fs::File;
fn main() {
    let x = "0.0001000";
    let prec = x.to_string().precision();
    println!("{}", prec);
    let y = 12.234879912;
    println!("{}", y.truncate(prec));

    let f = File::open("config.yml").unwrap();
    let configs: Value = serde_yaml::from_reader(f).unwrap();
    let x = configs
        .as_mapping()
        .unwrap()
        .get(&Value::String("base".to_string()))
        .unwrap()
        .as_mapping()
        .unwrap()
        .get(&Value::String("akey".to_string()))
        .unwrap().as_str().unwrap();
    println!("configs: {:?}", x);
}


#[cfg(test)]
mod test {
    use ba_api::client::rest_response::AggTrade;

  #[test]
  fn test_aggtrade() {
    let str = r##"
      {
        "e": "aggTrade",
        "E": 123456789, 
        "a": 12345,     
        "p": "0.001",   
        "q": "100",     
        "f": 100,       
        "l": 105,       
        "T": 123456785, 
        "m": true,      
        "M": true       
      }
    "##;

    let x = serde_json::from_str::<AggTrade>(str).unwrap();
    println!("{:?}", x);
  }
}
