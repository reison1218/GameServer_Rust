#[macro_use]
extern crate fluffy;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;

use actix_files::Files;
use actix_session::CookieSession;
use fluffy::db;

use std::time::{Duration, Instant};

use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, middleware};
use actix_web_actors::ws;


mod caches;
mod common;
mod config;
mod controllers;
mod filters;
mod models;
mod validations;
mod server;

use controllers::{
    admin_roles::AdminRoles, admins::Admins, ads::Ads, configs::Configs, index::Index,
    menus::Menus, navs::Navs, user_levels::UserLevels, users::Users, video_authors::VideoAuthors,
    video_categories::VideoCategories, video_replies::VideoReplies, video_tags::VideoTags,
    videos::Videos, watch_records::WatchRecords, Controller,
};


/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our route
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

struct WsChatSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// joined room
    room: String,
    /// peer name
    name: Option<String>,
    /// Chat server
    addr: Addr<server::ChatServer>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            self.addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_owned();
                                self.addr.do_send(server::Join {
                                    id: self.id,
                                    name: self.room.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                } else {
                    let msg = if let Some(ref name) = self.name {
                        format!("{}: {}", name, m)
                    } else {
                        m.to_owned()
                    };
                    // send message to chat server
                    self.addr.do_send(server::ClientMessage {
                        id: self.id,
                        msg,
                        room: self.room.clone(),
                    })
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl WsChatSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // 正式环境可以去掉日志显示
    std::env::set_var("RUST_LOG", "actix_web=info"); //正式环境可以注释此行 ***
    env_logger::init(); //正式环境可以注释此行 ***

    let setting = &*config::SETTING;
    let info = &setting.app;
    let conn_string = config::get_conn_string();
    db::init_connections(&conn_string); //資料庫初始化
    let host_port = &format!("{}:{}", &info.host, &info.port); //地址/端口
    println!("Started At: {}", host_port);

    let server = server::ChatServer::default().start();

    //let table_fields = caches::TABLE_FIELDS.lock().unwrap();
    HttpServer::new(move || {
        let mut tpl = tmpl!("/templates/**/*"); //模板引擎
        tpl.register_filter("state_name", filters::state_name);
        tpl.register_filter("menu_name", filters::menus::menu_name);
        tpl.register_filter("yes_no", filters::yes_no);
        tpl.register_filter("admin_role", filters::admin_roles::role_name);
        tpl.register_filter("position_name", filters::ads::position_name);
        tpl.register_filter("tag_name", filters::video_tags::tag_name);
        tpl.register_filter("author_name", filters::video_authors::author_name);

        //let generated = generate();
        App::new()
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .data(tpl)
            .data(server.clone())
            .wrap(middleware::Logger::default()) //正式环境可以注释此行 ***
            .service(Files::new("/static", "public/static/")) //静态文件目录
            .service(Files::new("/upload", "public/upload/")) //上传文件目录
            .service(web::resource("/test").to(Index::test))
            .service(get!("/", Index::index))
            .service(post!("/index/login", Index::login))
            .service(get!("/index/manage", Index::manage))
            .service(get!("/index/right", Index::right))
            .service(get!("/index/right", Index::right))
            .service(get!("/index/error", Index::error))
            .service(get!("/index/logout", Index::logout))
            .service(get!("/index/change_pwd", Index::change_pwd))
            .service(post!("/index/change_pwd_save", Index::change_pwd_save))
            .service(get!("/index/oss_signed_url", Index::oss_signed_url))
            .service(post!("/index/upload", Index::upload_images))
            //后台用户
            .service(get!("/admins", Admins::index))
            .service(get!("/admins/edit/{id}", Admins::edit))
            .service(post!("/admins/save/{id}", Admins::save))
            .service(get!("/admins/delete/{ids}", Admins::delete))
            //角色管理
            .service(get!("/admin_roles", AdminRoles::index))
            .service(get!("/admin_roles/edit/{id}", AdminRoles::edit))
            .service(post!("/admin_roles/save/{id}", AdminRoles::save))
            .service(get!("/admin_roles/delete/{ids}", AdminRoles::delete))
            //菜单管理
            .service(get!("/menus", Menus::index))
            .service(get!("/menus/edit/{id}", Menus::edit))
            .service(post!("/menus/save/{id}", Menus::save))
            .service(get!("/menus/delete/{ids}", Menus::delete))
            //前台用户
            .service(get!("/users", Users::index))
            .service(get!("/users/edit/{id}", Users::edit))
            .service(post!("/users/save/{id}", Users::save))
            .service(get!("/users/delete/{ids}", Users::delete))
            //视频分类
            .service(get!("/video_categories", VideoCategories::index))
            .service(get!("/video_categories/edit/{id}", VideoCategories::edit))
            .service(post!("/video_categories/save/{id}", VideoCategories::save))
            .service(get!(
                "/video_categories/delete/{ids}",
                VideoCategories::delete
            ))
            //视频管理
            .service(get!("/videos", Videos::index))
            .service(get!("/videos/edit/{id}", Videos::edit))
            .service(post!("/videos/save/{id}", Videos::save))
            .service(get!("/videos/delete/{ids}", Videos::delete))
            //视频标签
            .service(get!("/video_tags", VideoTags::index))
            .service(get!("/video_tags/edit/{id}", VideoTags::edit))
            .service(post!("/video_tags/save/{id}", VideoTags::save))
            .service(get!("/video_tags/delete/{ids}", VideoTags::delete))
            //视频作者
            .service(get!("/video_authors", VideoAuthors::index))
            .service(get!("/video_authors/edit/{id}", VideoAuthors::edit))
            .service(post!("/video_authors/save/{id}", VideoAuthors::save))
            .service(get!("/video_authors/delete/{ids}", VideoAuthors::delete))
            //用户等级
            .service(get!("/user_levels", UserLevels::index))
            .service(get!("/user_levels/edit/{id}", UserLevels::edit))
            .service(get!("/user_levels/delete/{ids}", UserLevels::delete))
            .service(post!("/user_levels/save/{id}", UserLevels::save))
            //观看记录
            .service(get!("/watch_records", WatchRecords::index))
            .service(get!("/watch_records/edit/{id}", WatchRecords::edit))
            .service(get!("/watch_records/delete/{ids}", WatchRecords::delete))
            .service(post!("/watch_records/save/{id}", WatchRecords::save))
            //replies
            .service(get!("/video_replies", VideoReplies::index))
            .service(get!("/video_replies/edit/{id}", VideoReplies::edit))
            .service(post!("/video_replies/save/{id}", VideoReplies::save))
            .service(get!("/video_replies/delete/{ids}", VideoReplies::delete))
            //广告管理
            .service(get!("/ads", Ads::index))
            .service(get!("/ads/edit/{id}", Ads::edit))
            .service(post!("/ads/save/{id}", Ads::save))
            .service(get!("/ads/delete/{ids}", Ads::delete))
            //网站导航
            .service(get!("/navs", Navs::index))
            .service(get!("/navs/edit/{id}", Navs::edit))
            .service(post!("/navs/save/{id}", Navs::save))
            .service(get!("/navs/delete/{ids}", Navs::delete))
            //网站设置
            .service(get!("/configs/edit/{id}", Configs::edit))
            .service(post!("/configs/save/{id}", Configs::save))
//            .service(web::resource("/ws/").route(web::get().to(websocket::ws_index)))
            .service(web::resource("/ws/").to(chat_route))
    })
        .bind(host_port)?
        .run()
        .await
}
