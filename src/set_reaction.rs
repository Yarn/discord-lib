
use hyper::{ Client, Request, Body };
use hyper::header::{ AUTHORIZATION, CONTENT_TYPE, CONTENT_LENGTH };
use percent_encoding::{ percent_encode, NON_ALPHANUMERIC };

use crate::discord::Snowflake;

use crate::send_message::Error;
use crate::send_message::send_retry_rate_limit;

pub async fn set_reaction<'a, C>(
    channel: Snowflake,
    msg: Snowflake,
    emoji: &str,
    base_url: &str,
    auth: &str,
    client: &'a Client<C, Body>,
    ) -> Result<(), Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    let Snowflake(channel_id) = channel;
    let Snowflake(msg_id) = msg;
    
    let emoji: String = percent_encode(emoji.as_bytes(), NON_ALPHANUMERIC).collect();
    
    let url = &format!("{}/channels/{}/messages/{}/reactions/{}/@me",
        base_url, channel_id, msg_id, emoji);
    
    send_retry_rate_limit(client, || {
        Request::builder()
            .method("PUT")
            .uri(url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, "0")
            .body(Body::empty())
            .map_err(|err| err.into())
    }).await?;
    
    Ok(())
}
