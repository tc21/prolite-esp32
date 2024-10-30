use std::sync::{mpsc::Sender, Arc, Mutex};

use esp_idf_svc::{
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    io::{EspIOError, Write},
    sys::EspError,
};

use crate::{
    protocol::{self, Animation, Command, Content},
    send,
};

mod api;

pub fn establish_control_server(
    sender: Sender<protocol::Command>,
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

            let sender = sender_ref.lock().unwrap();
            send(&sender, command);

            let response_content: String = "ok".to_owned(); // todo!();
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
        text: "Hello, World!".to_owned(),
        animation: Animation::None,
        duration: protocol::ContentDuration::Forever,
    })
}
