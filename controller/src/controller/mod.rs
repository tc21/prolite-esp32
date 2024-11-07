use std::sync::{Arc, Mutex};

use esp_idf_svc::{
    hal::uart::UartDriver, http::{
        server::{Configuration, EspHttpServer},
        Method,
    }, io::{EspIOError, Write}, sys::EspError
};


use log::info;
use prolite::api::{Animation, Color, Command, Content, ContentDuration, Repeat};

mod api;

pub fn establish_control_server(
    sender: UartDriver<'static>,
) -> Result<EspHttpServer<'static>, EspError> {
    // this code modified from https://github.com/esp-rs/std-training/blob/main/intro/http-server/examples/http_server.rs
    let mut server = EspHttpServer::new(&Configuration::default()).map_err(|e| e.0)?;

    // This only needed if we have more than one handler
    let sender = Arc::new(Mutex::new(sender));
    let sender_ref = Arc::clone(&sender);

    server.fn_handler(
        "/api/",
        // TODO figure out how post works
        Method::Get,
        move |request| -> core::result::Result<(), EspIOError> {
            let request_content: api::Request = api::Request {}; // todo!();
            let command: Command = parse_request(request_content);

            let serialize_and_send_result = match command.serialize() {
                Ok(s) => {
                    // if this fails it's probably better to restart the device
                    let sender = sender_ref.lock().unwrap();
                    Ok(sender.write(&s))
                },
                Err(e) => Err(e),
            };

            let response_content = match serialize_and_send_result {
                Ok(Ok(n)) => format!("written {} bytes", n),
                Ok(Err(e)) => format!("failed to send request: {:?}", e),
                Err(e) => format!("failed to serialize request: {:?}", e)
            };

            info!("[server] {}", response_content);

            let mut response = request.into_ok_response()?;
            response.write_all(response_content.as_bytes())?;
            Ok(())
        },
    )?;

    Ok(server)
}

fn parse_request(request: api::Request) -> Command {
    // todo!()
    Command::ShowNow(Content {
        text: "RT 550 X".to_owned(),
        color: Color::Orange,
        animation: Animation::None(ContentDuration::Forever),
        repeat: Repeat::None
    })
}
