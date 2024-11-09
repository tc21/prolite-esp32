use std::{net::Ipv4Addr, string::FromUtf8Error};

use esp_idf_svc::{
    hal::uart::UartDriver,
    http::{
        server::{Configuration, EspHttpConnection, EspHttpServer, Request},
        Method,
    },
    io::{EspIOError, Write},
    sys::EspError,
};

use log::info;
use prolite::api::{Color, Command, Content, ContentDuration, ContentGroup, Repeat};

pub fn establish_control_server(
    sender: UartDriver<'static>,
    ip_address: Ipv4Addr,
) -> Result<EspHttpServer<'static>, EspError> {
    // this code modified from https://github.com/esp-rs/std-training/blob/main/intro/http-server/examples/http_server.rs
    let mut server = EspHttpServer::new(&Configuration::default()).map_err(|e| e.0)?;

    // if this fails we will just restart
    let startup_command = Command::ShowNow {
        content: ContentGroup {
            contents: vec![Content {
                text: format!("IP: {}", ip_address),
                color: Color::default(),
                animation: prolite::api::Animation::None {
                    duration: ContentDuration::Forever,
                },
            }],
            repeat: Repeat::None,
        },
    };

    send_command(&startup_command, &sender).unwrap();

    server.fn_handler(
        "/api/",
        // TODO figure out how post works
        Method::Post,
        move |mut request| -> core::result::Result<(), EspIOError> {
            let response_content = match process_request(&mut request, &sender) {
                Ok(_) => "ok".to_owned(),
                Err(e) => format!("error: {}", e),
            };

            info!("[server] {}", response_content);

            let mut response = request.into_ok_response()?;
            response.write_all(response_content.as_bytes())?;
            Ok(())
        },
    )?;

    Ok(server)
}

fn process_request(
    request: &mut Request<&mut EspHttpConnection>,
    sender: &UartDriver<'static>,
) -> Result<(), String> {
    let request_content = match read_result(request) {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(format!("could not decode request: {}", e)),
        Err(e) => return Err(format!("could not read request: {}", e)),
    };

    let command = match serde_json::from_str::<prolite::api::Command>(&request_content) {
        Ok(c) => c,
        Err(e) => return Err(format!("could not parse request: {}", e)),
    };

    info!("parsed: {:?}", command);

    send_command(&command, sender)
}

fn send_command(command: &Command, sender: &UartDriver<'static>) -> Result<(), String> {
    let serialized_command = match serde_json::to_vec(command) {
        Ok(s) => s,
        Err(e) => return Err(format!("could not serialize request: {:?}", e)),
    };

    match sender.write(&serialized_command) {
        Ok(_) => { /* do nothing */ }
        Err(e) => return Err(format!("could not send request: {:?}", e)),
    }

    Ok(())
}

const BUFFER_SIZE: usize = 512;

fn read_result(
    request: &mut Request<&mut EspHttpConnection>,
) -> Result<Result<String, FromUtf8Error>, EspError> {
    let mut content = vec![];
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        match request.read(&mut buffer) {
            Ok(n) => {
                content.extend_from_slice(&buffer[..n]);
                if n < BUFFER_SIZE {
                    break;
                }
            }
            Err(e) => return Err(e.0),
        }
    }

    Ok(String::from_utf8(content))
}
