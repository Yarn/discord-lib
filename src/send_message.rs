
use std::borrow::Cow;

fn own_cow(cow: Cow<'_, str>) -> Cow<'static, str> {
    Cow::Owned(cow.into_owned())
}

#[derive(Debug, Serialize)]
struct NewMessageBorrowed<'a> {
    content: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Embed<'a> {
    title: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
}

impl<'a> Embed<'a> {
    fn into_owned(self) -> Embed<'static> {
        Embed {
            title: self.title.map(own_cow),
            description: self.description.map(own_cow),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum MentionTypes {
    Users,
    Roles,
    Everyone,
}

#[derive(Debug, Clone, Serialize)]
struct AllowedMentions {
    parse: Vec<MentionTypes>,
    users: Option<Vec<Snowflake>>,
    roles: Option<Vec<Snowflake>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewMessage<'a> {
    content: Option<Cow<'a, str>>,
    embed: Option<Embed<'a>>,
    allowed_mentions: Option<AllowedMentions>,
}

impl<'a> NewMessage<'a> {
    fn new(msg: String) -> Self {
        NewMessage {
            content: Some(msg.into()),
            embed: None,
            allowed_mentions: None,
        }
    }
    
    pub fn suppress_mentions(&mut self) {
        self.allowed_mentions = Some(AllowedMentions {
            parse: Vec::new(),
            users: None,
            roles: None,
        });
    }
    
    pub fn embed_desc<T1, T2>(title: Option<T1>, msg: T2) -> Self
    // pub fn embed_desc<T>(title: Option<T>, msg: T) -> Self
        where 
            T1: Into<Cow<'a, str>>,
            T2: Into<Cow<'a, str>>,
        // where Cow<'a, str>: From<T>
    {
        let temp: Cow<'a, str> = msg.into();
        
        NewMessage {
            content: None,
            embed: Some(Embed {
                title: title.map(|x| x.into()),
                // title: None,
                // description: Some(msg.into()),
                description: Some(temp),
            }),
            allowed_mentions: None,
        }
    }
    
    pub fn embed_temp(msg: String) -> Self {
        NewMessage {
            content: None,
            embed: Some(Embed {
                title: None,
                description: Some(msg.into()),
            }),
            allowed_mentions: None,
        }
    }
    
    pub fn into_owned(self) -> NewMessage<'static> {
        // let content = Cow::Owned(self.content.map(|x| x.into_owned()).unwrap_or("".into()));
        // let content = self.content.map(own_cow);
        NewMessage {
            content: self.content.map(own_cow),
            embed: self.embed.map(|embed| embed.into_owned()),
            allowed_mentions: self.allowed_mentions,
        }
    }
}

impl From<&str> for NewMessage<'_> {
    fn from(x: &str) -> Self {
        NewMessage::new(x.to_string())
    }
}

impl From<String> for NewMessage<'static> {
    fn from(x: String) -> Self {
        NewMessage::new(x)
    }
}

// use futures::compat::Future01CompatExt;
// use futures01::Stream;
use tokio::time::delay_for;
use hyper_tls;
use hyper_tls::HttpsConnector;
use crate::discord::Snowflake;
use hyper;
use hyper::Body;
use hyper::Request;
use hyper::Client;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper::http::StatusCode;
use serde_json;
use serde::Deserialize;

pub use hyper::Error as HyperError;
pub use hyper::http::Error as HyperHttpError;
pub use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    DiscordError{
        msg: String,
        status_code: StatusCode,
        body: String,
    },
    TransportError(hyper::Error),
    HttpError(hyper::http::Error),
    DecodeError(std::string::FromUtf8Error),
    RateLimited {
        retry_after: u64,
    },
    Other(String)
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::TransportError(err)
    }
}
impl From<hyper::http::Error> for Error {
    fn from(err: hyper::http::Error) -> Self {
        Error::HttpError(err)
    }
}
impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::DecodeError(err)
    }
}
// impl From<hyper_tls::Error> for Error {
//     fn from(err: hyper_tls::Error) -> Self {
//         let reason = format!("TLS Error: {}", err);
//         Error::Other(reason)
//     }
// }

pub type Https = HttpsConnector<hyper::client::HttpConnector>;
// pub type Https = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;
pub type TheClient = Client<Https, Body>;

pub fn get_https() -> Result<Https, Error> {
    Ok(HttpsConnector::new())
        // .map_err(|err| err.into())
    // Ok(hyper_rustls::HttpsConnector::new())
}

pub fn get_client() -> Result<TheClient, Error> {
    let https = get_https()?;
    
    let client = Client::builder()
        .build(https);
    
    Ok(client)
}

#[derive(Debug, Deserialize)]
pub struct DiscordRateLimitResponse {
    retry_after: f64,
    global: bool,
}

pub(crate) async fn send_retry_rate_limit<'a, C, ReqF>(client: &'a Client<C, Body>, req_builder: ReqF) -> Result<String, Error> 
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
        ReqF: Fn() -> Result<Request<Body>, Error> + 'a,
{
    loop {
        let req = req_builder()?;
        
        let res = client.request(req).await?;
        
        let (parts, body) = res.into_parts();
        
        let body = {
            let b: Vec<u8> = hyper::body::to_bytes(body).await?.to_vec();
            
            String::from_utf8(b)?
        };
        
        if parts.status == StatusCode::TOO_MANY_REQUESTS {
            let res: DiscordRateLimitResponse = serde_json::from_str(&body)
                .map_err(|err| Error::Other(format!("Malformed rate limit message {}", err)))?;
            
            if res.global {
                dbg!(&res);
            }
            
            // delay_for(::std::time::Duration::from_millis(res.retry_after)).await;
            delay_for(::std::time::Duration::from_secs_f64(res.retry_after)).await;
            continue
        }
        
        if !parts.status.is_success() {
            return Err(Error::DiscordError {
                msg: format!("Status Code {:?}", parts.status),
                status_code: parts.status,
                body: body,
            });
        }
        
        break Ok(body)
    }
}

fn split_msg(msg: &str) -> Vec<Cow<str>> {
    if msg.len() > 2000 {
        let mut new_out = vec![Cow::Owned(String::new())];
        let split = msg.lines();
        
        let mut last_line_long = false;
        for line in split {
            if last_line_long {
                last_line_long = false;
                new_out.push("".into());
            }
            
            if line.len() > 2000 {
                last_line_long = true;
                if new_out.last().unwrap() != "" {
                    new_out.push("".into());
                }
                
                for c in line.chars() {
                    if new_out.last().unwrap().len() >= 2000 {
                        new_out.push("".into());
                    }
                    
                    new_out.last_mut().unwrap().to_mut().push(c);
                }
                
                continue
            }
            
            // add 1 for \n added if false
            if new_out.last().unwrap().len() + line.len() + 1 > 2000 {
                new_out.push(Cow::Owned("".into()));
            } else {
                new_out.last_mut().unwrap().to_mut().push('\n');
            }
            new_out.last_mut().unwrap().to_mut().push_str(line);
        }
        
        for msg in new_out.iter_mut() {
            if msg.trim() == "" {
                msg.to_mut().push('.')
            }
        }
        
        new_out
    } else {
        vec![Cow::Borrowed(msg)]
    }
}

async fn send_inner<'a, C>(msg: &'a NewMessage<'a>, auth: &str, url: &str, client: &'a Client<C, Body>) -> Result<(), Error>
        where
            C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
    {
    
    let _body = send_retry_rate_limit(client, || {
        let body: Body = serde_json::to_string(&msg)
            .map_err(|err| Error::Other(format!("Could not serialize message {}", err)))?
            .into();
        
        Request::builder()
            .method("POST")
            .uri(url)
            
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/json")
            // .body(Body::empty())
            .body(body)
            .map_err(|err| err.into())
    }).await?;
    
    Ok(())
}

pub async fn send<'a, C>(to: Snowflake, msg: &'a NewMessage<'a>, base_url: &'a str, token: &'a str, client: &'a Client<C, Body>) -> Result<(), Error>
    where
        C: hyper::client::connect::Connect + 'static + Clone + Send + Sync,
{
    // https://discordapp.com/api/v6/gateway/bot
    
    let Snowflake(to_id) = to;
    
    let url = &format!("{}/channels/{}/messages", base_url, to_id);
    
    let auth = &format!("Bot {}", token);
    
    // let mut msgs = Vec::new();
    
    let full_msg = msg;
    match (&msg.content, &msg.embed) {
        (Some(content), None) => {
            for msg in split_msg(content) {
                // let ref msg = NewMessageBorrowed {
                //     content: &*msg,
                // };
                
                let msg = NewMessage {
                    content: msg.into(),
                    embed: None,
                    allowed_mentions: full_msg.allowed_mentions.clone(),
                };
                
                send_inner(&msg, auth, url, client).await?;
            }
        }
        _ => {
            send_inner(msg, auth, url, client).await?;
        }
    }
    
    // for msg in msgs {
    //     send_inner(&msg, auth, url, client).await?;
    // }
    
    Ok(())
}

