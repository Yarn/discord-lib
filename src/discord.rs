
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
