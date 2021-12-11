use std::path::Path;
use serde_yaml;

struct ApiConfig {
  access_key: String,
  sercet_key: String,
  log_file: Path, 
}



