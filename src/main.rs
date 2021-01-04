use chrono::NaiveDate;
use dotenv::dotenv;
use std::collections::HashMap;
use std::error::Error;
use structopt::StructOpt;

mod model;
mod serve;

extern crate base64;
extern crate dotenv;

fn parse_date(src: &str) -> NaiveDate {
    NaiveDate::parse_from_str(&src, &model::DATE_FORMAT).unwrap()
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// Simple license server
#[derive(StructOpt, Debug)]
#[structopt(name = "license-server")]
enum Opt {
    /// Generate a license
    Generate {
        /// Start date of the license
        #[structopt(long, short = "f", parse(from_str=parse_date))]
        valid_from: Option<NaiveDate>,
        /// End date of the license
        #[structopt(long, short = "u", parse(from_str=parse_date))]
        valid_until: Option<NaiveDate>,
        /// Metadata to add to the license.
        #[structopt(short = "M", parse(try_from_str = parse_key_val), number_of_values = 1)]
        meta: Vec<(String, String)>,
        /// HMAC Signing Secret
        #[structopt(short = "s", long = "secret", env = "HMAC_SECRET")]
        secret: String,
    },
    /// Validate a license
    Validate {
        /// License string
        #[structopt()]
        license: String,
        /// HMAC Signing Secret
        #[structopt(short = "s", long = "secret", env = "HMAC_SECRET")]
        secret: String,
    },
    /// Start the license server
    Serve {
        /// License string
        #[structopt(short = "k", long = "key", env = "LICENSE_API_KEY")]
        api_key: String,
        /// HMAC Signing Secret
        #[structopt(short = "s", long = "secret", env = "HMAC_SECRET")]
        secret: String,
        /// Listen port
        #[structopt(short = "p", long = "port", env = "PORT", default_value = "3000")]
        port: u16,
    },
}

fn generate(
    secret: String,
    valid_from: Option<NaiveDate>,
    valid_until: Option<NaiveDate>,
    meta: Vec<(String, String)>,
) -> Result<(), String> {
    let actual_from = valid_from.unwrap_or_else(|| chrono::offset::Utc::now().naive_utc().date());
    let actual_to = valid_until.unwrap_or_else(|| actual_from + chrono::Duration::days(30));
    let mut m: HashMap<String, String> = HashMap::new();
    meta.iter().for_each(|e| {
        m.insert(e.0.to_string(), e.1.to_string());
    });
    let license = model::License {
        id: uuid::Uuid::new_v4().to_string(),
        meta: m,
        valid_from: actual_from.format(model::DATE_FORMAT).to_string(),
        valid_until: actual_to.format(model::DATE_FORMAT).to_string(),
    };
    let result = license.hash(secret)?;
    println!("Your license key is:\n{:}", result);
    Ok(())
}

fn validate(secret: String, license: String) -> Result<(), String> {
    let sl = model::SignedLicense::new(&license)?;
    sl.validate(secret)?;
    println!("Your license key is:\n{:}\nValid!\n", &license);
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    if let Err(e) = match Opt::from_args() {
        Opt::Generate {
            secret,
            valid_from,
            valid_until,
            meta,
        } => generate(secret, valid_from, valid_until, meta),
        Opt::Validate { secret, license } => validate(secret, license),
        Opt::Serve {
            secret,
            api_key,
            port,
        } => {
            serve::serve(secret, api_key, port).await;
            Ok(())
        }
    } {
        println!("❌ {}", e)
    }
}
