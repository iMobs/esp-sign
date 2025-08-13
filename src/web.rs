use crate::WEB_TASK_POOL_SIZE;
use embassy_time::Duration;
use picoserve::{
    extract::Json,
    make_static,
    routing::get_service,
    AppBuilder, AppRouter, Router,
};
use rgb::ComponentSlice;

pub struct Application;

impl AppBuilder for Application {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new().route(
            "/",
            get_service(picoserve::response::File::html(include_str!("index.html")))
                .post(set_color),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
struct FormData {
    color: heapless::String<32>,
}

async fn set_color(Json(data): Json<FormData>) -> Json<&'static str> {
    let mut rgb = rgb::RGB8::default();
    hex::decode_to_slice(&data.color[1..], rgb.as_mut_slice()).unwrap();
    defmt::info!("Setting RGB to: {:?}", rgb);
    crate::RGB_CHANNEL.send(rgb).await;
    Json("ok")
}

pub async fn init_web(stack: embassy_net::Stack<'static>, spawner: &embassy_executor::Spawner) {
    let web_app = WebApp::default();

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(id, stack, web_app.router, web_app.config));
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<Application>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}

pub struct WebApp {
    pub router: &'static Router<<Application as AppBuilder>::PathRouter>,
    pub config: &'static picoserve::Config<Duration>,
}

impl Default for WebApp {
    fn default() -> Self {
        let router = make_static!(AppRouter<Application>, Application.build_app());

        let config = make_static!(
            picoserve::Config::<Duration>,
            picoserve::Config::new(picoserve::Timeouts {
                start_read_request: Some(Duration::from_secs(5)),
                persistent_start_read_request: Some(Duration::from_secs(1)),
                read_request: Some(Duration::from_secs(1)),
                write: Some(Duration::from_secs(1)),
            })
            .keep_connection_alive()
        );

        Self { router, config }
    }
}
