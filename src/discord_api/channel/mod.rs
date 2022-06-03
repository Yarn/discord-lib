
use serde::de::IgnoredAny;
use serde_repr::{Serialize_repr, Deserialize_repr};
use crate::discord::double_option;

use hyper::{ Client, Request, Body };
use hyper::header::{ AUTHORIZATION, CONTENT_TYPE, CONTENT_LENGTH };

use crate::discord::{
    Snowflake,
    User,
};

use crate::send_message::Error;
use crate::send_message::send_retry_rate_limit;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum ChannelType {
    GuildText = 0,
    Dm = 1,
    GuildVoice = 2,
    GroupDm = 3,
    GuildCatagory = 4,
    GuildNews = 5,
    GuildStore = 6,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Channel {
    pub id: Snowflake,
    r#type: ChannelType,
    pub guild_id: Option<Snowflake>,
    position: Option<isize>,
    permission_overwrites: Option<IgnoredAny>,
    name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    topic: Option<Option<String>>,
    nsfw: Option<bool>,
    #[serde(default, deserialize_with = "double_option")]
    last_message_id: Option<Option<Snowflake>>,
    bitrate: Option<isize>,
    user_limit: Option<isize>,
    rate_limit_per_user: Option<isize>,
    recipients: Option<Vec<User>>,
    #[serde(default, deserialize_with = "double_option")]
    icon: Option<Option<String>>,
    owner_id: Option<Snowflake>,
    application_id: Option<Snowflake>,
    #[serde(default, deserialize_with = "double_option")]
    parent_id: Option<Option<Snowflake>>,
    #[serde(default, deserialize_with = "double_option")]
    last_pin_timestamp: Option<Option<IgnoredAny>>,
}

pub async fn get_channel<'a, C>(
    channel: Snowflake,
    // msg: Snowflake,
    // emoji: &str,
    base_url: &str,
    auth: &str,
    client: &'a Client<C, Body>,
    ) -> Result<Channel, Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    let Snowflake(channel_id) = channel;
    // let Snowflake(msg_id) = msg;
    
    // let emoji: String = percent_encode(emoji.as_bytes(), NON_ALPHANUMERIC).collect();
    
    let url = &format!("{}/channels/{}",
        base_url, channel_id);
    
    let body = send_retry_rate_limit(client, || {
        Request::builder()
            .method("GET")
            .uri(url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, "0")
            .body(Body::empty())
            .map_err(|err| err.into())
    }).await?;
    
    let channel: Channel = serde_json::from_str(&body)
        .map_err(|err| Error::Other(format!("Malformed get channel {}", err)))?;
    
    Ok(channel)
}
