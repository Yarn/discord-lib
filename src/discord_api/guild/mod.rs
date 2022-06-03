
use hyper::{ Client, Request, Body };
use hyper::header::{ AUTHORIZATION, CONTENT_TYPE, CONTENT_LENGTH };

use crate::discord::{
    Snowflake,
    GuildMember,
};

use crate::send_message::Error;
use crate::send_message::send_retry_rate_limit;

pub mod roles;

const LIMIT: usize = 1000;

async fn get_members_inner<'a, C>(
    server: Snowflake,
    after: Option<Snowflake>,
    // msg: Snowflake,
    // emoji: &str,
    base_url: &str,
    auth: &str,
    client: &'a Client<C, Body>,
    ) -> Result<Vec<GuildMember>, Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    let Snowflake(server_id) = server;
    
    let mut url = format!("{}/guilds/{}/members?limit={}",
        base_url, server_id, LIMIT);
    
    if let Some(Snowflake(after)) = after {
        url.push_str(&format!("&after={}", after))
    }
    
    let body = send_retry_rate_limit(client, || {
        Request::builder()
            .method("GET")
            .uri(&url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, "0")
            .body(Body::empty())
            .map_err(|err| err.into())
    }).await?;
    
    // println!("{}", body);
    let users = serde_json::from_str(&body)
        .map_err(|err| Error::Other(format!("Malformed get members {}", err)))?;
    
    Ok(users)
}

pub async fn get_members<'a, C>(
    server: Snowflake,
    // msg: Snowflake,
    // emoji: &str,
    base_url: &str,
    auth: &str,
    client: &'a Client<C, Body>,
    ) -> Result<Vec<GuildMember>, Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    let mut users = Vec::new();
    let mut last = None;
    
    loop {
        let data = get_members_inner(server.clone(), last.clone(), base_url, auth, client).await?;
        
        let is_end = data.len() < LIMIT;
        
        if let Some(max) = data.iter().filter_map(|x| x.user.as_ref()).map(|x| x.id.0).max() {
            last = Some(Snowflake(max))
        }
        
        for user in data {
            users.push(user)
        }
        
        if is_end {
            break
        }
    }
    
    Ok(users)
}
