use crate::web::start_web;
use anyhow::Result;
use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

mod web;

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
    println!("current dir: {:?}", std::env::current_dir());

    let web_ip = "127.0.0.1";
    let web_port = 7000;
    let wry_begin_url = format!("http://{}:{}/index.html", web_ip, web_port);

    // 启动 web 服务
    std::thread::spawn(move || {
        start_web(web_ip, web_port);
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello World")
        .build(&event_loop)
        .unwrap();
    window.set_maximized(true);

    let webview = WebViewBuilder::new(window)
        .unwrap()
        .with_url(wry_begin_url.as_str())?
        .build()?;

    log::info!("current dir: {:?}", std::env::current_dir().unwrap());

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry application started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {
                let _ = webview.resize();
            }
        }
    });
}
