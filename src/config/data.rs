use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use slog::Logger;
use std::str::FromStr;

use mpl_candy_machine::{
    EndSettingType as CandyEndSettingType, EndSettings as CandyEndSettings,
    GatekeeperConfig as CandyGatekeeperConfig, HiddenSettings as CandyHiddenSettings,
    WhitelistMintMode as CandyWhitelistMintMode,
    WhitelistMintSettings as CandyWhitelistMintSettings,
};
pub struct SugarConfig {
    pub logger: Logger,
    pub keypair: Keypair,
    pub rpc_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigData {
    pub price: f64,

    pub number: u64,

    pub gatekeeper: Option<GatekeeperConfig>,

    #[serde(rename = "solTreasuryAccount")]
    #[serde(deserialize_with = "to_pubkey")]
    pub sol_treasury_account: Pubkey,

    #[serde(rename = "splTokenAccount")]
    #[serde(deserialize_with = "to_option_pubkey")]
    pub spl_token_account: Option<Pubkey>,

    #[serde(rename = "splToken")]
    #[serde(deserialize_with = "to_option_pubkey")]
    pub spl_token: Option<Pubkey>,

    #[serde(rename = "goLiveDate")]
    pub go_live_date: String,

    #[serde(rename = "endSettings")]
    pub end_settings: Option<EndSettings>,

    #[serde(rename = "whitelistMintSettings")]
    pub whitelist_mint_settings: Option<WhitelistMintSettings>,

    #[serde(rename = "hiddenSettings")]
    pub hidden_settings: Option<HiddenSettings>,

    #[serde(rename = "uploadMethod")]
    pub upload_method: UploadMethod,

    #[serde(rename = "retainAuthority")]
    pub retain_authority: bool,

    #[serde(rename = "isMutable")]
    pub is_mutable: bool,
}

pub fn go_live_date_as_timestamp(go_live_date: &str) -> Result<i64> {
    let go_live_date = chrono::DateTime::parse_from_rfc3339(go_live_date)?;
    Ok(go_live_date.timestamp())
}

pub fn price_as_lamports(price: f64) -> u64 {
    (price * LAMPORTS_PER_SOL as f64) as u64
}

fn to_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}

fn to_option_pubkey<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = match Deserialize::deserialize(deserializer) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    let pubkey = Pubkey::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(Some(pubkey))
}

fn discount_price_to_lamports(discount_price: Option<f64>) -> Option<u64> {
    if let Some(discount_price) = discount_price {
        Some((discount_price * LAMPORTS_PER_SOL as f64) as u64)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatekeeperConfig {
    /// The network for the gateway token required
    gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    expire_on_use: bool,
}

impl GatekeeperConfig {
    pub fn into_candy_format(&self) -> CandyGatekeeperConfig {
        CandyGatekeeperConfig {
            gatekeeper_network: self.gatekeeper_network,
            expire_on_use: self.expire_on_use,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EndSettingType {
    Date,
    Amount,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndSettings {
    end_setting_type: EndSettingType,
    number: u64,
}

impl EndSettings {
    pub fn into_candy_format(&self) -> CandyEndSettings {
        CandyEndSettings {
            end_setting_type: match self.end_setting_type {
                EndSettingType::Date => CandyEndSettingType::Date,
                EndSettingType::Amount => CandyEndSettingType::Amount,
            },
            number: self.number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhitelistMintSettings {
    mode: WhitelistMintMode,
    #[serde(deserialize_with = "to_pubkey")]
    mint: Pubkey,
    presale: bool,
    #[serde(rename = "discountPrice")]
    discount_price: Option<f64>,
}

impl WhitelistMintSettings {
    pub fn into_candy_format(&self) -> CandyWhitelistMintSettings {
        CandyWhitelistMintSettings {
            mode: self.mode.into_candy_format(),
            mint: self.mint,
            presale: self.presale,
            discount_price: discount_price_to_lamports(self.discount_price),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WhitelistMintMode {
    BurnEveryTime,
    NeverBurn,
}

impl WhitelistMintMode {
    pub fn into_candy_format(&self) -> CandyWhitelistMintMode {
        match self {
            WhitelistMintMode::BurnEveryTime => CandyWhitelistMintMode::BurnEveryTime,
            WhitelistMintMode::NeverBurn => CandyWhitelistMintMode::NeverBurn,
        }
    }
}

impl FromStr for WhitelistMintMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "burneverytime" => Ok(WhitelistMintMode::BurnEveryTime),
            "neverburn" => Ok(WhitelistMintMode::NeverBurn),
            _ => Err(anyhow::anyhow!("Invalid whitelist mint mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HiddenSettings {
    name: String,
    uri: String,
    hash: [u8; 32],
}

impl HiddenSettings {
    pub fn into_candy_format(&self) -> CandyHiddenSettings {
        CandyHiddenSettings {
            name: self.name.clone(),
            uri: self.uri.clone(),
            hash: self.hash.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UploadMethod {
    Metaplex,
    Bundlr,
    Arloader,
}

impl FromStr for UploadMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "metaplex" => Ok(UploadMethod::Metaplex),
            "bundlr" => Ok(UploadMethod::Bundlr),
            "arloader" => Ok(UploadMethod::Arloader),
            _ => Err(format!("Unknown storage: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for UploadMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}