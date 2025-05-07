
use serde::{
    Serialize, Serializer,
    Deserialize, Deserializer,
};
use serde::de;

type No = serde::de::IgnoredAny;

pub(crate) fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
    where T: Deserialize<'de>,
          D: Deserializer<'de>
{
    Deserialize::deserialize(de).map(Some)
}

fn de_num_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    u64::from_str_radix(&s, 10).map_err(de::Error::custom)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snowflake(pub u64);

impl std::ops::Deref for Snowflake {
    type Target = u64;
    
    fn deref(&self) -> &Self::Target {
        let Snowflake(x) = self;
        x
    }
}

impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

// TODO: make seperate SnowflakeVisitor
impl<'de> de::Visitor<'de> for Snowflake {
    type Value = Self;
    
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string representing a 64-bit int")
    }
    
    fn visit_str<E>(self, v: &str) -> Result<Self, E>
        where E: de::Error
    {
        let num = v.parse().map_err(|_| {
            E::invalid_value(de::Unexpected::Str(v), &"")
        })?;
        
        Ok(Snowflake(num))
        // Ok(num)
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
        where D: Deserializer<'de>
    {
        deserializer.deserialize_str(Snowflake(0))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Role {
    pub id: Snowflake,
    name: String,
    #[serde(deserialize_with = "de_num_str")]
    pub permissions: u64,
    pub position: usize,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GuildMember {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub roles: Vec<Snowflake>,
    joined_at: No,
    premium_since: Option<No>,
    deaf: bool,
    mute: bool,
    permissions: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub author: User,
    pub member: Option<GuildMember>,
    pub content: String,
    // timestamp: No,
    #[serde(rename = "timestamp")]
    pub timestamp_str: String,
    edited_timestamp: No,
    tts: bool,
    mention_everyone: bool,
    pub mentions: Vec<User>, // User + extra data?
    // pub mention_roles: Vec<Role>,
    pub mention_roles: Vec<Snowflake>,
    attachments: Vec<No>,
    embeds: Vec<No>,
    reactions: Option<Vec<No>>,
    nonce: Option<Snowflake>,
    pinned: bool,
    webhook_id: Option<Snowflake>,
    #[serde(rename="type")]
    message_type: u64,
    activity: Option<No>,
    application: Option<No>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Guild {
    pub id: Snowflake,
    name: String,
    pub owner_id: Snowflake,
    pub roles: Vec<Role>,
    // only present for GUILD_CREATE
    #[serde(default)]
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub enum OverwriteType {
    Role,
    Member,
    Unknown,
}

fn _de_overwrite_type<'de, D>(deserializer: D) -> Result<OverwriteType, D::Error>
    where D: Deserializer<'de>
{
    let x = usize::deserialize(deserializer)?;
    match x {
        0 => Ok(OverwriteType::Role),
        1 => Ok(OverwriteType::Member),
        _ => Ok(OverwriteType::Unknown),
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Overwrite {
    pub id: Snowflake,
    #[serde(rename = "type", deserialize_with = "_de_overwrite_type")]
    pub overwrite_type: OverwriteType,
    #[serde(deserialize_with = "de_num_str")]
    pub allow: u64,
    #[serde(deserialize_with = "de_num_str")]
    pub deny: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Channel {
    pub id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub permission_overwrites: Option<Vec<Overwrite>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct UnavailableGuild {
    pub id: Snowflake,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DeletedMessage {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreatedRole {
    pub guild_id: Snowflake,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DeletedRole {
    pub guild_id: Snowflake,
    pub role_id: Snowflake,
}
