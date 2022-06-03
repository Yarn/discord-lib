
// use bytes::Bytes;

use hyper::{Body, Request, StatusCode};
use hyper::header::{UPGRADE, CONNECTION};

use futures::Future;

use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Role;
pub use tokio_tungstenite::tungstenite::Message;
// use futures01::{Stream, future};
// use futures::compat::Future01CompatExt;
use futures::stream::StreamExt;

// use futures::compat::Stream01CompatExt;
use futures::TryFutureExt;
use futures::SinkExt;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

pub use crate::send_message::TheClient;

#[derive(Debug, Fail)]
pub enum WebSocketError {
    #[fail(display = "Connection Failed: {}", reason)]
    Jank {
        reason: String,
    },
    #[fail(display = "NoUpgrade: {}", status)]
    NoUpgrade {
        status: StatusCode,
        body: Vec<u8>,
    },
}

impl WebSocketError {
    fn jank(reason: String) -> Self {
        let jank = WebSocketError::Jank {
            reason: reason,
        };
        jank
    }
}

type WSStream = WebSocketStream<hyper::upgrade::Upgraded>;
// type TGItem = Result<tungstenite::Message, tungstenite::Error>;
// type TGNext = Option<TGItem>;
type WSItem = Result<Message, WebSocketError>;
// type WSNext = Option<WSItem>;

// type Sender = UnboundedSender<String>;
pub type SenderM = UnboundedSender<Message>;
type SenderRM = UnboundedSender<WSItem>;
// type Recv = UnboundedReceiver<String>;
type RecvM = UnboundedReceiver<Message>;
type RecvRM = UnboundedReceiver<WSItem>;

#[derive(Debug)]
pub struct WebSocket {
    pub sender: SenderM,
    receiver: RecvRM,
}

async fn ws_into_io_task(ws: WSStream, mut out_msgs_recv: RecvM, mut in_msg_chan: SenderRM) {
    // use futures::StreamExt;
    // use futures::SinkExt;
    use futures::select;
    // use futures01::Stream;
    // use futures01::Sink;
    
    let (mut sink, stream) = ws.split();
    // let mut stream = stream.compat();
    let mut stream = stream;
    
    let mut out_msg = out_msgs_recv.next().fuse();
    let mut in_msg = stream.next().fuse();
    loop {
        // println!("before select");
        let _res = select! {
            out_msg_val = out_msg => {
                // println!("\nout_msg: {:?}", out_msg);
                
                if let Some(msg) = out_msg_val {
                    // eprintln!("CCC {:?}", msg);
                    // eprintln!("-- ws send");
                    // ws.send(msg.into());
                    let fut = (&mut sink).send(msg.into());
                    // fut.compat().await.unwrap();
                    fut.await.unwrap();
                } else {
                    eprintln!("\n\n!!!out channel ended!!!\n\n");
                    break;
                }
                
                out_msg = out_msgs_recv.next().fuse();
                
                false
            },
            in_msg_val = in_msg => {
                // dbg!(&in_msg_val);
                // eprintln!("in msg{:?}", in_msg_val);
                // dbg_wrapper(in_msg_val);
                if let Some(Err(err)) = in_msg_val {
                    eprintln!("ws err: {}", err);
                    break;
                }
                if let None = in_msg_val {
                    eprintln!("ws closed: ");
                    break;
                }
                
                if let Some(Ok(msg)) = in_msg_val {
                    if let Err(err) = in_msg_chan.send(Ok(msg)).await {
                        eprintln!("ws io task, send channel closed: {}", err);
                        break
                    }
                }
                
                in_msg = stream.next().fuse();
                
                true
            },
        };
    }
}

impl WebSocket {
    
    pub fn send<'a, T: Into<Message>>(&'a mut self, msg: T) -> impl Future<Output = Result<(), WebSocketError>> + 'a {
        self.sender.send(msg.into()).map_err(|err| {
            WebSocketError::jank(format!("mpsc send error in WebSocket::send : {:?}", err))
        })
    }
    
    pub fn recv<'a>(&'a mut self) -> impl Future<Output = Result<Message, WebSocketError>> + 'a {
        async move {
            let fut = self.receiver.next();
            
            let msg: Option<WSItem> = fut.await;
            let msg = match msg {
                Some(Ok(msg)) => msg,
                Some(Err(err)) => return Err(err),
                None => return Err(WebSocketError::jank("msg recv channel is closed".into()))
            };
            
            Ok(msg)
        }
    }
}

pub struct WebSocketBuilder {
    url: String,
}

impl WebSocketBuilder {
    pub fn new(url: String) -> Self {
        
        Self {
            url: url,
        }
    }
    
    pub fn init<'a>(self, client: &'a TheClient) -> impl Future<Output = Result<WebSocket, WebSocketError>> + 'a {
        async move {
            let ws: WSStream = open_websocket(&self.url, client).await?;
            
            let (out_msgs_send, out_msgs_recv) = futures::channel::mpsc::unbounded::<Message>();
            let (in_msgs_send, in_msgs_recv) = futures::channel::mpsc::unbounded::<WSItem>();
            
            let io_task = ws_into_io_task(ws, out_msgs_recv, in_msgs_send);
            
            jank_spawn(io_task);
            
            Ok(WebSocket{
                sender: out_msgs_send,
                receiver: in_msgs_recv,
            })
        }
    }
}

pub fn jank_spawn(fut: impl Future<Output = ()> + Send + 'static) {
    // tokio::spawn(fut.unit_error().boxed().compat());
    tokio::spawn(fut.unit_error().boxed());
}
pub fn jank_run(_fut: impl Future<Output=()> + Send + 'static) {
    // tokio::run(fut.unit_error().boxed().compat());
    // tokio::run(fut.unit_error().boxed());
    panic!("jank_run roke");
}

use futures::FutureExt;

async fn open_websocket<'a>(url: &'a str, client: &'a TheClient) -> Result<WSStream, WebSocketError> {
    // use futures01::Future;
    // use futures::Future;
    
    // let res = client.get("https://hyper.rs".parse().unwrap()).await.unwrap();
    // assert_eq!(res.status(), 200);
    
    
    
    // get a 404 with wss url, something is probably broken in hyper
    // other programs work with wss:// url for the same server
    let url = String::from(url);
    let url = url.replace("wss://", "https://");
    let url = url.replace("ws://", "http://");
    // println!("uri internal {}", url);
    // let url = "https://gateway.discord.gg:443/a?v=6&encoding=json";
    // let url = "https://localhost:3001/?v=6&encoding=json";
    // let url = "http://localhost:3001/?v=6&encoding=json";
    // let url = "https://echo.websocket.org";
    // let url = "https://hyper.rs";
    // let url = "https://discordapp.com";
    
    use hyper::Version;
    
    let req = Request::builder()
        .uri(url)
        
        .version(Version::HTTP_11)
        
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "Upgrade")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("Sec-WebSocket-Version", "13")
        
        .body(Body::empty())
        .map_err(|err| WebSocketError::jank(format!("request builder error: {}", err)))?;
    
    // println!("{:?}", req);
    // let client = crate::send_message::get_client().unwrap();
    
    let fut = client
        .request(req)
        .map_err(|e| { WebSocketError::jank(format!(".request error: {}", e)) });
    
    let res = fut.await?;
    
    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
        let status = res.status();
        
        // let mut body = hyper::body::aggregate(res)
        let mut body = hyper::body::to_bytes(res)
            .map_err(|e| WebSocketError::jank(format!("body error: {}", e)))
            .await?;
        use hyper::body::Buf;
        // let b: &[u8] = a.bytes();
        
        let err = WebSocketError::NoUpgrade {
            status: status,
            // body: a.to_bytes(),
            body: body.to_bytes().to_vec(),
        };
        
        return Err(err);
        // let e = "not impl";
        // return Err(WebSocketError::jank(format!(".into_body(),concat2() error: {}", e)));
    }
    
    let upgraded = res
        .into_body()
        .on_upgrade()
        .map_err(|e| { WebSocketError::jank(format!("on upgrade error: {}", e)) })
        .await?;
    
    // panic!()
    // let upgraded = fut.await.unwrap();
    let websocket = WebSocketStream::from_raw_socket(upgraded, Role::Client, None);
    
    // // fut.compat().await
    // // fut.await
    // // fut
    Ok(websocket.await)
}
