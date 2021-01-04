use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use hmac::{Hmac, Mac, NewMac};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct License {
    pub(crate) id: Option<String>,
    pub(crate) meta: HashMap<String, String>,
    pub(crate) valid_from: String,
    pub(crate) valid_until: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SignedLicense {
    pub(crate) license: License,
    pub(crate) signature: String,
}

impl SignedLicense {
    pub fn validate(&self, secret: String) -> Result<(), &'static str> {
        let from = NaiveDate::parse_from_str(&self.license.valid_from, &DATE_FORMAT)
            .map_err(|_| "invalid date format")?;
        let until = NaiveDate::parse_from_str(&self.license.valid_until, &DATE_FORMAT)
            .map_err(|_| "invalid date format")?;
        let now = Utc::now().naive_utc().date();
        if now < from || now > until {
            return Err("License expired or not yet active");
        }
        let sig = self.license.sign(secret)?;
        if sig == self.signature {
            Ok(())
        } else {
            Err("Invalid signature!")
        }
    }
    pub fn new(encoded: &str) -> Result<Self, &'static str> {
        let decoded_hash = base64::decode(encoded).map_err(|_| "invalid hash")?;
        let z = std::str::from_utf8(&decoded_hash).map_err(|_| "invalid hash")?;
        let n = serde_json::from_str(z).map_err(|_| "cannot decode json")?;
        Ok(n)
    }
}

impl License {
    pub fn sign(&self, secret: String) -> Result<String, &'static str> {
        let mut parts: Vec<String> = vec![];
        parts.push(self.valid_from.to_string());
        parts.push(self.valid_until.to_string());
        parts.push(self.id.as_ref().expect("license missing id").to_string());
        self.meta.iter().for_each(|e| parts.push(e.1.to_string()));
        let to_hash = parts.join("\n");
        let mut mac = HmacSha256::new_varkey(secret.as_bytes()).map_err(|_| "invalid hash")?;
        mac.update(to_hash.as_bytes());
        let code = mac.finalize().into_bytes();
        Ok(code
            .iter()
            .format_with("", |byte, f| f(&format_args!("{:02x}", byte)))
            .to_string())
    }

    pub fn hash(&self, secret: String) -> Result<String, &'static str> {
        let sig = self.sign(secret)?;
        let signed = SignedLicense {
            license: self.clone(),
            signature: sig,
        };

        let result = serde_json::to_string(&signed).map_err(|_| "cannot encode")?;
        Ok(base64::encode(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::model;
    const SECRET: &str = "SECRET";
    const VALID_HASH: &str = "eyJsaWNlbnNlIjp7Im1hY2hpbmUiOm51bGwsIm1ldGEiOnt9LCJ2YWxpZF9mcm9tIjoiMS0yLTMiLCJ2YWxpZF91bnRpbCI6IjEtMi00In0sInNpZ25hdHVyZSI6IjljMzY2ZjQ3YTg0MjVjMzc2ZjAyOGJkMzk4MDViYzQxMTYwZDY3MzQxMGIxODg4NTRhYjJhZmViMDFmMDEzZTEifQ";
    #[test]
    fn test_generate() {
        let license = model::License {
            id: None,
            meta: Default::default(),
            valid_from: "1-2-3".to_string(),
            valid_until: "1-2-4".to_string(),
        };
        let hash = license.hash(SECRET.to_string()).unwrap();
        assert_eq!(hash, VALID_HASH);
    }

    #[test]
    fn test_validate() {
        // let validate_path = api_validate(SECRET.to_string()).recover(handle_rejection);
        // let resp = request()
        //     .method("POST")
        //     .path("/validate/test")
        //     .body(&TEST_LICENSE_REQUEST)
        //     .reply(&validate_path)
        //     .await;
        //
        // assert_eq!(resp.status(), StatusCode::OK);
    }
}
