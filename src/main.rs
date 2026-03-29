#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;

use std::{cell::RefCell, error::Error, rc::Rc};
use async_compat::Compat;
use slint::{ModelRc, SharedString, VecModel};

use crate::server::Message;

slint::include_modules!();

impl From<Message> for UIMessage {
    fn from(value: Message) -> Self {
        Self {
            author: value.author.into(),
            content: value.content.into()
        }
    }
}

struct ConfigStateThing {
    roomname: SharedString,
    username: SharedString,
    ip: SharedString,
    connected: bool
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = LcolachatWindow::new()?;

    let messages_vec_model: Rc<VecModel<UIMessage>> = Rc::from(VecModel::from(vec![]));
    let config_state = Rc::from(RefCell::from(ConfigStateThing {
        roomname: SharedString::new(),
        username: SharedString::new(),
        ip: SharedString::new(),
        connected: false
    }));

    let req_client = Rc::new(reqwest::ClientBuilder::new().build().unwrap());

    ui.on_host({
        let ui = ui.as_weak().unwrap();
        let cs2 = Rc::clone(&config_state);

        move |room, user| {
            ui.set_menu_state(MenuState::Loading);

            let mut cs2 = cs2.borrow_mut();

            ui.set_chat_name(room.clone());

            cs2.roomname = room.clone();
            cs2.username = user;
            cs2.ip = "localhost".into();
            cs2.connected = true;

            slint::spawn_local({
                let ui = ui.as_weak();

                Compat::new(async move {
                    server::start_server(room.into(), ui.unwrap()).await;
                })
            }).unwrap();

            ui.set_menu_state(MenuState::Messages);
        }
    });

    ui.on_send_message({
        let messages_vec_model = Rc::clone(&messages_vec_model);
        let ui = ui.as_weak().unwrap();
        let req_client = Rc::clone(&req_client);
        let config_state = Rc::clone(&config_state);

        move |content| {
            let messages_vec_model = Rc::clone(&messages_vec_model);
            let ui = ui.as_weak().unwrap();
            let req_client = Rc::clone(&req_client);
            let config_state = Rc::clone(&config_state);

            slint::spawn_local(Compat::new(async move {
                let ip = config_state.borrow().ip.clone();
                let author = config_state.borrow().username.clone();

                let req = req_client.post(format!("http://{ip}:3621/message"))
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&Message {
                        author: author.into(),
                        content: content.into()
                    }).unwrap())
                    .send().await;

                if let Ok(req) = req {
                    if req.status().as_u16() == 200 {
                        let req_txt = req.text().await.unwrap();
                        let messages: Vec<Message> = serde_json::from_str(&req_txt).unwrap();

                        messages_vec_model.clear();

                        for message in messages {
                            println!("h {}", message.content);
                            messages_vec_model.push(message.into());
                        }

                        ui.set_messages(ModelRc::from(messages_vec_model));
                    } else {
                        ui.set_menu_state(MenuState::Error);
                    }
                } else {
                    ui.set_menu_state(MenuState::Error);
                }
            })).unwrap();
        }
    });

    ui.run()?;

    Ok(())
}
