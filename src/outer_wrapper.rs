
use futures::Future;

pub use crate::send_message::TheClient;
pub use crate::send_message;
pub use crate::gateway::{Gateway, GatewayBuilder, GatewayMessage, GatewayError};
pub use crate::discord::{Snowflake, Message, User, GuildMember};
pub use crate::send_message::NewMessage;
use crate::set_reaction;
use crate::discord_api::channel::get_channel;
use crate::Channel;

use crate::gateway::GatewayMessage as GM;
use crate::gateway::Event as E;

#[derive(Debug)]
pub struct Discord {
    client: TheClient,
    gateway: Gateway,
    base_url: String,
    token: String,
    session_id: Option<String>,
}

impl Discord {
    pub fn connect(base_url: String, token: String) -> impl Future<Output=Result<Self, ()>> {
        async move {
            let http_client = crate::send_message::get_client().unwrap();
            
            let gateway: Gateway = GatewayBuilder::new().base_url(base_url.clone()).connect(token.clone(), &http_client).await.unwrap();
            
            Ok(Self {
                client: http_client,
                gateway: gateway,
                base_url: base_url,
                token: token,
                session_id: None,
            })
        }
    }
    
    pub fn send<'a>(&'a self, to: Snowflake, msg: &'a NewMessage) -> impl Future<Output=Result<(), send_message::Error>> + 'a {
        send_message::send(to, msg, &self.base_url, &self.token, &self.client)
    }
    
    pub async fn send_gateway_raw(&mut self, msg: &str) -> Result<(), ()> {
        self.gateway.ws.send(msg).await.unwrap();
        
        // Err(())
        Ok(())
    }
    
    pub fn get_send_handle<'a>(&'a self) -> SendHandle {
        SendHandle {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            token: self.token.clone(),
            auth: format!("Bot {}", self.token),
        }
    }
    
    pub fn recv<'a>(&'a mut self) -> impl Future<Output=Result<GatewayMessage, GatewayError>> + 'a {
        async move {
            loop {
                let msg = self.gateway.recv().await;
                
                match msg {
                    Ok(GM::Event(E::Ready(ref ready))) => {
                        self.session_id = Some(ready.session_id.clone());
                    }
                    Ok(GM::InvalidSession) => {
                        // panic!("op 9 (Invalid Session) not supported");
                        self.session_id = None;
                        self.gateway.seq_num = None;
                        break Err(GatewayError::InvalidSession)
                    }
                    Ok(GM::Reconnect) => {
                        // match (self.session_id, self.gateway.seq_num) {
                        //     (Some(ref session_id), Some(seq) => {
                        //         // let seq: u64 = self.gateway.seq_num.expect("Resume before seq number");
                                
                        //         let gateway: Gateway = GatewayBuilder::new()
                        //             .base_url(self.base_url.clone())
                        //             .resume(session_id.clone(), seq)
                        //             .connect(self.token.clone(), &self.client).await?;
                                
                        //         self.gateway = gateway
                        //     }
                        //     _ => {
                        //         // panic!("received reconnect (op 7) before Ready")
                        //         return GatewayError::Misc("".into())
                        //     }
                        // }
                        let is_reconnect = self.reconnect().await?;
                        if is_reconnect {
                            eprintln!("Reconnect recieved before session_id or sequence number")
                        }
                    }
                    _ => {}
                }
                
                break msg
            }
        }
    }
    
    pub async fn reconnect_with(base_url: String, token: String, session_id: String, seq: u64) -> Result<Self, GatewayError> {
        let http_client = crate::send_message::get_client().unwrap();
        
        let gateway: Gateway = GatewayBuilder::new()
            .base_url(base_url.clone())
            .resume(session_id.clone(), seq)
            .connect(token.clone(), &http_client).await?;
        // gateway.seq_num = Some(seq);
        
        Ok(Self {
            client: http_client,
            gateway: gateway,
            base_url: base_url,
            token: token,
            session_id: Some(session_id),
        })
    }
    
    pub fn reconnect<'a>(&'a mut self) -> impl Future<Output=Result<bool, GatewayError>> + 'a {
        async move {
            let mut is_reconnect = false;
            
            let mut builder = GatewayBuilder::new()
                .base_url(self.base_url.clone());
            
            if let (Some(ref session_id), Some(seq)) = (&self.session_id, self.gateway.seq_num) {
                is_reconnect = true;
                
                // let seq: u64 = .expect("Resume before seq number");
                
                builder = builder.resume(session_id.clone(), seq);
            }
            
            let gateway: Gateway = builder.connect(self.token.clone(), &self.client).await?;
            
            self.gateway = gateway;
            
            // match self.session_id {
            //     Some(ref session_id) => {
                    
            //     }
            //     None => {
            //         // panic!("received reconnect (op 7) before Ready")
            //     }
            // }
            Ok(is_reconnect)
        }
    }
    
    pub fn seq(&self) -> Option<u64> {
        self.gateway.seq_num
    }
    
    pub fn did_resume(&self) -> Option<bool> {
        self.gateway.did_resume
    }
}

#[derive(Debug, Clone)]
pub struct SendHandle {
    client: TheClient,
    base_url: String,
    token: String,
    auth: String,
}

impl SendHandle {
    pub fn send<'a>(&'a self, to: Snowflake, msg: &'a NewMessage) -> impl Future<Output=Result<(), send_message::Error>> + 'a {
        send_message::send(to, msg, &self.base_url, &self.token, &self.client)
    }
    
    pub async fn set_reaction(&self, channel: Snowflake, msg: Snowflake, emoji: &str) -> Result<(), send_message::Error> {
        set_reaction::set_reaction(channel, msg, emoji, &self.base_url, &self.auth, &self.client).await
    }
    
    pub async fn get_channel(&self, channel: Snowflake) -> Result<Channel, send_message::Error> {
        get_channel(channel, &self.base_url, &self.auth, &self.client).await
    }
    
    pub async fn get_guild_members(&self, guild: Snowflake) -> Result<Vec<GuildMember>, send_message::Error> {
        crate::discord_api::guild::get_members(
            guild, &self.base_url, &self.auth, &self.client
        ).await
    }
    
    pub async fn add_member_role(&self, guild: Snowflake, user: Snowflake, role: Snowflake) -> Result<(), send_message::Error> {
        crate::discord_api::guild::roles::add_member_role(
            guild, user, role, &self.base_url, &self.auth, &self.client
        ).await
    }
    
    pub async fn remove_member_role(&self, guild: Snowflake, user: Snowflake, role: Snowflake) -> Result<(), send_message::Error> {
        crate::discord_api::guild::roles::remove_member_role(
            guild, user, role, &self.base_url, &self.auth, &self.client
        ).await
    }
}
