
use reqwest::header::AUTHORIZATION;

use futures::Future;

use serde_json;

use crate::gateway_ws;
pub use crate::send_message::TheClient;

use crate::gateway_ws::WebSocketError;

#[derive(Debug, Deserialize)]
pub struct GatewayInfo {
    #[serde(rename = "url")]
    ws_url: String,
}

impl GatewayInfo {
    pub fn new(url: String) -> Self {
        Self {
            ws_url: url
        }
    }
    
    pub fn get_url<'a>(&'a self) -> &'a str {
        &self.ws_url
    }
}

#[derive(Debug, Fail)]
pub enum GatewayError {
    #[fail(display = "Gateway Misc: {}", 0)]
    Misc(String),
    #[fail(display = "Web Socket Error: {}", 0)]
    WebSocket(WebSocketError),
    #[fail(display = "Malformed Payload Error")]
    MalformedPayload,
    #[fail(display = "Request Error {}", 0)]
    ReqwestError(reqwest::Error),
    #[fail(display = "Invalid Session")]
    InvalidSession,
}

impl From<WebSocketError> for GatewayError {
    fn from(err: WebSocketError) -> Self {
        GatewayError::WebSocket(err)
    }
}

impl From<ParsePayloadError> for GatewayError {
    fn from(err: ParsePayloadError) -> Self {
        match err {
            ParsePayloadError::Malformed => GatewayError::MalformedPayload,
            _ => GatewayError::Misc(format!("Unexpected Parse Payload Error: {:?}", err))
        }
    }
}

impl From<reqwest::Error> for GatewayError {
    fn from(err: reqwest::Error) -> Self {
        GatewayError::ReqwestError(err)
    }
}

async fn get_gateway<'a>(token: &'a str, base_url: &'a str, client: &'a TheClient) -> Result<GatewayInfo, GatewayError> {
    let mut auth = String::from("Bot ");
    auth.push_str(token);
    
    let url = format!("{}/gateway/bot", base_url);
    
    //let addr: ::std::net::SocketAddr = ([127, 0, 0, 1], 8080).into();
    // let addr: ::std::net::SocketAddr = "https://discordapp.com/api/v6/gateway/bot".parse().unwrap();
    //https://discordapp.com/api
    
    let req = client.get(url)
        .header(AUTHORIZATION, auth)
        .build()
        .unwrap();
    
    let res: reqwest::Response = client.execute(req).await?;
    
    let a = res.bytes().await?;
    let b: &[u8] = a.as_ref();
    
    let info: GatewayInfo = serde_json::from_slice(b)
        .map_err(|err| {
            eprintln!("deserialize error (get_gateway): {}", err); GatewayError::Misc("deserialize".into());
            let s = String::from_utf8_lossy(b);
            eprintln!("response (get_gateway) {}", s);
            GatewayError::Misc("deserialize".into())
        })?;
    
    Ok(info)
}

use self::gateway_ws::{WebSocket, WebSocketBuilder, Message};
use self::gateway_ws::SenderM;
use serde_json::value::Value;
use serde_json::from_value;
use crate::discord;

#[derive(Debug)]
pub enum Event {
    MessageCreate(discord::Message),
    MessageReactionAdd(MessageReactionAdd),
    MessageReactionRemove(MessageReactionRemove),
    Ready(Ready),
    VoiceStateUpdate(VoiceStateUpdate),
    VoiceServerUpate(VoiceServerUpdate),
    PresenceUpdate(PresenceUpdate),
    Unknown(String, Value),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u64,
    #[serde(rename = "_trace")]
    trace: Vec<String>,
}

// BBBBBB {"t":"PRESENCE_UPDATE",
// "s":107210,
// "op":0,
// "d":{
//     "user":{"id":"-"},
//     "status":"online",
//     "roles":["-"],
//     "guild_id":"-",
//     "game":{
//         "type":4,"name":"Custom Status","id":"custom",
//         "emoji":{"name":"-","id":"-","animated":true
//        },
//        "created_at":-
//      },
//     "client_status":{"desktop":"online"},
//     "activities":[
        // {"type":4,"name":"Custom Status","id":"custom","emoji":{"name":"-","id":"-","animated":true},
//      "created_at":-}]}}

use crate::discord::{
    Snowflake,
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct IdUser {
    id: Snowflake,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PresenceUpdate {
    user: IdUser,
    status: String,
    roles: Vec<Snowflake>,
    guild_id: Snowflake,
}

#[derive(Debug)]
pub enum GatewayMessage {
    // MessageCreate(Value),
    Event(Event),
    Hello(Hello),
    Reconnect,
    InvalidSession,
    Unknown(Value),
    Raw(Message),
    Temp
}

#[derive(Debug, Deserialize)]
struct Payload {
    op: u64,
    #[serde(default, rename = "d")]
    data: Value,
    #[serde(rename = "s")]
    seq_num: Option<u64>,
    #[serde(rename = "t")]
    event: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Emoji {
    id: Option<Snowflake>,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MessageReactionAdd {
    pub user_id: Snowflake,
    pub channel_id: Snowflake,
    pub message_id: Snowflake,
    guild_id: Option<Snowflake>,
    member: Option<discord::GuildMember>,
    pub emoji: Emoji,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MessageReactionRemove {
    pub user_id: Snowflake,
    pub channel_id: Snowflake,
    pub message_id: Snowflake,
    guild_id: Option<Snowflake>,
    pub emoji: Emoji,
}

#[derive(Debug, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
}

#[derive(Debug, Deserialize)]
pub struct Ready {
    pub user: discord::User,
    pub session_id: String,
    #[serde(rename = "v")]
    pub protocol_version: usize,
    pub guilds: Vec<UnavailableGuild>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct VoiceStateUpdate {
    session_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct VoiceServerUpdate {
    endpoint: String,
    token: String,
    guild_id: String,
}

#[derive(Debug, Clone)]
struct ResumeInfo {
    session_id: String,
    seq: u64,
}

// pub struct Gateway<S> {
#[derive(Debug)]
pub struct Gateway {
    pub ws: WebSocket,
    token: String,
    resume: Option<ResumeInfo>,
    pub(crate) seq_num: Option<u64>,
    pub(crate) did_resume: Option<bool>,
    // pub spawner: S,
}

use self::gateway_ws::jank_spawn;

#[derive(Debug)]
enum ParsePayloadError {
    Unkown,
    Malformed
}

fn parse_payload_inner(payload: Payload) -> Result<Option<GatewayMessage>, ParsePayloadError> {
    let out_msg: GatewayMessage = match payload.op {
        0 => {
            match payload.event.as_ref().map(|s| s.as_str()) {
                Some("MESSAGE_CREATE") => {
                    let message: discord::Message = from_value(payload.data).map_err(|err| {
                        eprintln!("parsing event failed for op 0 MESSAGE_CREATE: {:?}", err);
                        ParsePayloadError::Malformed
                    })?;
                    GatewayMessage::Event(Event::MessageCreate(message))
                }
                Some("READY") => {
                    let ready: Ready = from_value(payload.data).map_err(|err| {
                        eprintln!("parsing event failed for op 0 READY: {:?}", err);
                        ParsePayloadError::Malformed
                    })?;
                    GatewayMessage::Event(Event::Ready(ready))
                }
                Some("VOICE_STATE_UPDATE") => {
                    // {
                    //     "t":"VOICE_STATE_UPDATE",
                    //     "s":4,"op":0,
                    //     "d":{
                    //         "member":{
                    //             "user":{"username":"ndirc","id":"-","discriminator":"-","bot":true,"avatar":null},
                    //             "roles":[],"mute":false,"joined_at":"2017-08-30T10:42:45.869000+00:00",
                    //             "hoisted_role":null,"deaf":false
                    //         },
                    //         "user_id":"-","suppress":false,
                    //         "session_id":"-",
                    //         "self_video":false,"self_mute":false,"self_deaf":false,"mute":false,
                    //         "guild_id":"-","deaf":false,"channel_id":"-"
                    //     }
                    // }
                    
                    let session_id = payload.data
                        .get("session_id").expect("VOICE_STATE_UPDATE session id missing")
                        .as_str().expect("VOICE_STATE_UPDATE session id not string");
                    
                    let obj = VoiceStateUpdate {
                        session_id: session_id.into(),
                    };
                    
                    GatewayMessage::Event(Event::VoiceStateUpdate(obj))
                }
                Some("VOICE_SERVER_UPDATE") => {
                    // {
                    //     "t":"VOICE_SERVER_UPDATE","s":5,"op":0,
                    //     "d":{
                    //         "token":"-",
                    //         "guild_id":"-",
                    //         "endpoint":"-"
                    //     }
                    // }
                    
                    let update: VoiceServerUpdate = from_value(payload.data).map_err(|err| {
                        eprintln!("parsing voice server update failed for op 0 VOICE_SERVER_UPDATE: {:?}", err);
                        ParsePayloadError::Malformed
                    })?;
                    
                    GatewayMessage::Event(Event::VoiceServerUpate(update))
                }
                Some("PRESENCE_UPDATE") => {
                    let update: PresenceUpdate = from_value(payload.data).map_err(|err| {
                        eprintln!("parsing presence update failed PRESENCE_UPDATE: {:?}", err);
                        ParsePayloadError::Malformed
                    })?;
                    
                    GatewayMessage::Event(Event::PresenceUpdate(update))
                }
                Some("MESSAGE_REACTION_ADD") => {
                    let data: MessageReactionAdd = from_value(payload.data).map_err(|err| {
                        eprintln!("parsing failed MESSAGE_REACTION_ADD: {:?}", err);
                        ParsePayloadError::Malformed
                    })?;
                    
                    GatewayMessage::Event(Event::MessageReactionAdd(data))
                }
                Some("MESSAGE_REACTION_REMOVE") => {
                    let data: MessageReactionRemove = from_value(payload.data).map_err(|err| {
                        eprintln!("parsing failed MESSAGE_REACTION_REMOVE: {:?}", err);
                        ParsePayloadError::Malformed
                    })?;
                    
                    GatewayMessage::Event(Event::MessageReactionRemove(data))
                }
                Some(event_type) => {
                    GatewayMessage::Event(Event::Unknown(event_type.into(), payload.data))
                }
                _ => {
                    return Err(ParsePayloadError::Unkown)
                }
            }
        }
        7 => {
            GatewayMessage::Reconnect
        }
        9 => {
            GatewayMessage::InvalidSession
        }
        10 => {
            let hello: Hello = from_value(payload.data).map_err(|err| {
                eprintln!("parsing hello failed {:?}", err);
                ParsePayloadError::Malformed
            })?;
            
            GatewayMessage::Hello(hello)
        }
        11 => {
            // ping ack
            return Ok(None);
        }
        _ => {
            return Err(ParsePayloadError::Unkown)
        }
    };
    
    return Ok(Some(out_msg))
}

fn parse_payload(payload: Payload) -> (Result<Option<GatewayMessage>, ParsePayloadError>, Option<u64>) {
    let seq_num = payload.seq_num;
    let inner = parse_payload_inner(payload);
    (inner, seq_num)
}

const IDENTIFY_TEMPLATE: &'static str = r#"{
    "op": 2,
    "d": {
        "token": "TOKEN",
        "intents": <intents>,
        "properties": {"$os": "linux", "$browser": "test", "$device": "test"}
    }
}"#;

#[derive(Serialize)]
struct ResumeData<'a> {
    token: &'a str,
    session_id: &'a str,
    seq: u64,
}
#[derive(Serialize)]
struct Resume<'a> {
    op: u8,
    d: ResumeData<'a>,
}

impl<'a> Resume<'a> {
    fn new(token: &'a str, session_id: &'a str, seq: u64) -> Self {
        Resume {
            op: 6,
            d: ResumeData {
                token,
                session_id,
                seq,
            },
        }
    }
}

impl Gateway {
    
    fn connect<'a>(token: String, base_url: String, client: &'a TheClient, resume: Option<ResumeInfo>) -> impl Future<Output = Result<Self, GatewayError>> + 'a {
        async move {
            
            let gateway_info = get_gateway(&token, &base_url, &client).await?;
            
            // dbg!("gateway: {:?}", gateway_info);
            
            let url: &str = gateway_info.get_url();
            let mut url = String::from(url);
            if !url.ends_with("/") {
                url.push_str("/")
            }
            url.push_str("?v=9&encoding=json");
            let url: &str = &url;
            
            let builder = WebSocketBuilder::new(String::from(url));
            
            let ws = builder.init(client).await?;
            
            let gateway = Gateway {
                ws: ws,
                token: token,
                resume: resume,
                seq_num: None,
                did_resume: None,
                // spawner: spawner,
            };
            
            Ok(gateway)
        }
    }
    
    pub fn recv<'a>(&'a mut self) -> impl Future<Output = Result<GatewayMessage, GatewayError>> + 'a {
        async move {
            loop {
                // dbg!("before recv");
                let msg: Message = self.ws.recv().await.map_err(|err| {GatewayError::Misc(format!("failed to recv message: {}", err))})?;
                // eprintln!("AAAAA {:?}", msg);
                
                if let Message::Text(ref text) = msg {
                    // eprintln!("BBBBBB {}", text);
                    
                    let payload: Payload = serde_json::from_str(text)
                        .map_err(|err| {
                            eprintln!("Could not parse gateway msg {:?} {:?}", err, text);
                            GatewayError::MalformedPayload
                        })?;
                    
                    let (payload, seq) = parse_payload(payload);
                    if let Some(_) = seq {
                        self.seq_num = seq
                    }
                    
                    let gw_msg: GatewayMessage = match payload {
                        Ok(Some(msg)) => {
                            msg
                        }
                        Ok(None) => continue,
                        Err(ParsePayloadError::Unkown) => {
                            GatewayMessage::Raw(msg)
                        }
                        Err(ParsePayloadError::Malformed) => {
                            return Err(GatewayError::MalformedPayload)
                        }
                    };
                    
                    if let GatewayMessage::Hello(ref hello) = gw_msg {
                        let heartbeat_task = send_heartbeat(hello.heartbeat_interval, self.ws.sender.clone());
                        jank_spawn(heartbeat_task);
                        
                        match self.resume {
                            Some(ref resume_info) => {
                                self.did_resume = Some(true);
                                let resume = Resume::new(&self.token, &resume_info.session_id, resume_info.seq);
                                
                                let msg: String = serde_json::to_string(&resume).unwrap();
                                
                                self.ws.send(msg).await.map_err(|err| {
                                    GatewayError::Misc(format!("failed to respond to hello (resume) {:?}", err))
                                })?;
                                // panic!()
                            }
                            None => {
                                self.did_resume = Some(false);
                                let intents: usize = 
                                    (1 << 0 ) + // GUILDS
                                    (1 << 9 ) + // GUILD_MESSAGES
                                    (1 << 10) + // GUILD_MESSAGE_REACTIONS
                                    (1 << 12) + // DIRECT_MESSAGES
                                    (1 << 13) // DIRECT_MESSAGE_REACTIONS
                                    ;
                                let resp: String = IDENTIFY_TEMPLATE.replace("TOKEN", &self.token).replace("<intents>", &intents.to_string());
                                self.ws.send(resp).await.map_err(|err| {
                                    GatewayError::Misc(format!("failed to respond to hello {:?}", err))
                                })?;
                            }
                        }
                    }
                    
                    break Ok(gw_msg)
                } else {
                    break Ok(GatewayMessage::Raw(msg))
                }
            }
        }
    }
}

async fn send_heartbeat(interval_ms: u64, mut sender: SenderM) {
    // use futures01::IntoFuture;
    use futures::SinkExt;
    // use tokio::time::delay_for;
    
    loop {
        let sec: u64 = interval_ms / 1000;
        let ms: u32 = (interval_ms % 1000) as u32;
        
        // tokio_timer::sleep(::std::time::Duration::new(sec, ms)).into_future().compat().await.unwrap();
        // delay_for(::std::time::Duration::new(sec, ms)).await;
        ::tokio::time::sleep(::std::time::Duration::new(sec, ms)).await;
        // println!("heartbeat");
        if let Err(err) = sender.send("{\"op\": 1, \"d\": null}".into()).await {
            eprintln!("heartbeat, send channel closed: {}", err);
            break
        }
    }
}

pub struct GatewayBuilder {
    base_url_val: String,
    resume: Option<ResumeInfo>,
}

impl GatewayBuilder {
    pub fn new() -> Self {
        GatewayBuilder {
            base_url_val: "https://discordapp.com/api/v9".into(),
            resume: None,
        }
    }
    
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url_val = base_url;
        self
    }
    
    pub fn resume(mut self, session_id: String, seq: u64) -> Self {
        self.resume = Some(ResumeInfo { session_id, seq });
        self
    }
    
    pub fn connect<'a>(self, token: String, client: &'a TheClient) -> impl Future<Output = Result<Gateway, GatewayError>> + 'a {
        Gateway::connect(token, self.base_url_val, client, self.resume)
    }
}
