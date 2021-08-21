use actix::prelude::*;
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::web::Bytes;
use actix_web::{get, Error, Result};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use log::info;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

#[get("/{filename:.*}")]
pub async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path = Path::new("web/public").join(path);
    Ok(NamedFile::open(path)?)
}

/*
    定义 websocket 行为
*/

#[get("/ws")]
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    log::debug!("{:?}", &req);
    let resp = ws::start(VideoWs {}, &req, stream);
    log::debug!("{:?}", &resp);
    resp
}

pub struct VideoWs {}

impl Actor for VideoWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(256);

        let addr = ctx.address();

        thread::spawn(move || {
            let mut i = 0;
            let time_per_frame = 1f32 / 24f32; // 24 frames per second
            let now = Instant::now();
            while addr.connected() {
                let start_decode = now.elapsed();
                if i >= 100 {
                    i = 0;
                }
                i += 1;

                let mut buf = Vec::new();
                let filename = format!("yuv/video_encode_yuv420_{}.bin", i);
                let size = File::open(filename).unwrap().read_to_end(&mut buf).unwrap();
                log::info!("size: {}", size);

                let data = Bytes::copy_from_slice(buf.as_slice());

                let msg = DataMsg::binary(data);
                addr.do_send(msg);

                let current = (now.elapsed() - start_decode).as_secs_f32();
                let delay = time_per_frame - current;
                if delay > 0.0 {
                    // tokio::time::sleep(Duration::from_secs_f32(delay)).await;
                    sleep(Duration::from_secs_f32(delay));
                } else {
                    log::info!(
                        "per frame time: {}, delay: {}",
                        time_per_frame * 1000.0,
                        delay * 1000.0
                    );
                }
            }
        });
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for VideoWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                ctx.text(format!("!!! unknown command: {:?}", &text));
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl Handler<DataMsg> for VideoWs {
    type Result = ();
    fn handle(&mut self, msg: DataMsg, ctx: &mut Self::Context) -> Self::Result {
        if let Some(binary) = msg.binary {
            ctx.binary(binary);
        }
        if let Some(text) = msg.text {
            ctx.text(text);
        }
    }
}

pub struct DataMsg {
    pub binary: Option<Bytes>,
    pub text: Option<String>,
}
impl Message for DataMsg {
    type Result = ();
}

impl DataMsg {
    pub fn binary(data: Bytes) -> Self {
        Self {
            binary: Some(data),
            text: None,
        }
    }
    pub fn text(data: String) -> Self {
        Self {
            binary: None,
            text: Some(data),
        }
    }
}

pub fn start_web(ip: &str, port: u16) {
    info!("start web service");
    let system = System::new();
    system.block_on(async {
        let web_addr = format!("{}:{}", ip, port);
        log::debug!("web_addr: {:?}", &web_addr);
        HttpServer::new(move || {
            App::new()
                .service(ws_index)
                .service(index)
                .wrap(Cors::permissive())
        })
        .bind(web_addr)
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    system.run().unwrap();
}
