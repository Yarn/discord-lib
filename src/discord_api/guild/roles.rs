
use hyper::{ Client, Request, Body };
use hyper::header::{ AUTHORIZATION, CONTENT_TYPE, CONTENT_LENGTH };

use crate::discord::Snowflake;

use crate::send_message::Error;
use crate::send_message::send_retry_rate_limit;

pub async fn add_member_role<'a, C>(
    guild_id: Snowflake,
    user_id: Snowflake,
    role_id: Snowflake,
    // after: Option<Snowflake>,
    // msg: Snowflake,
    // emoji: &str,
    base_url: &str,
    auth: &str,
    client: &'a Client<C, Body>,
    ) -> Result<(), Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    let url = format!("{}/guilds/{}/members/{}/roles/{}",
        base_url, guild_id.0, user_id.0, role_id.0);
    
    let _body = send_retry_rate_limit(client, || {
        Request::builder()
            .method("PUT")
            .uri(&url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, "0")
            .body(Body::empty())
            .map_err(|err| err.into())
    }).await?;
    
    Ok(())
}

pub async fn remove_member_role<'a, C>(
    guild_id: Snowflake,
    user_id: Snowflake,
    role_id: Snowflake,
    // after: Option<Snowflake>,
    // msg: Snowflake,
    // emoji: &str,
    base_url: &str,
    auth: &str,
    client: &'a Client<C, Body>,
    ) -> Result<(), Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    let url = format!("{}/guilds/{}/members/{}/roles/{}",
        base_url, guild_id.0, user_id.0, role_id.0);
    
    let _body = send_retry_rate_limit(client, || {
        Request::builder()
            .method("DELETE")
            .uri(&url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, "0")
            .body(Body::empty())
            .map_err(|err| err.into())
    }).await?;
    
    Ok(())
}
