extern crate config;

#[macro_use]
extern crate rustful;
extern crate rustc_serialize;
extern crate unicase;
extern crate postgres;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate uuid;

use std::path::Path;
use std::error::Error;
use std::str::FromStr;

use unicase::UniCase;


use rustful::{Server, Context, Response, TreeRouter, Handler, Method};
use rustful::server::Host;
use rustful::header::{
    ContentType,
    AccessControlAllowOrigin,
    AccessControlAllowMethods,
    AccessControlAllowHeaders,
};
use postgres::Connection;
use postgres::TlsMode;
use rustful::StatusCode;
use rustc_serialize::json;
use config::reader::from_file;
use config::types::Config;

#[macro_use]
mod structs;

mod admin;
mod assignment;

fn main() {
    let config = from_file(Path::new("config/current.conf")).expect("Could not find current.conf!");

    env_logger::init().unwrap();

    let server_result = Server {
        host: Host::from_str(config.lookup_str("port").unwrap_or("0.0.0.0:8080")).unwrap(),
        handlers: insert_routes!{
            TreeRouter::new() => {
                "admin" => {
                    Get: Api(Some(admin::get_santa)),
                    Post: Api(Some(admin::post_santa)),
                    Delete: Api(Some(admin::delete_santa))
                },
                "assignment/:code" => {
                    Get: Api(Some(assignment::get_assignment))
                }
            }
        },
        global: Box::new(config).into(),
        ..Server::default()
    }.run();

    match server_result {
        Ok(_server) => {},
        Err(e) => error!("Could not start server: {}", e.description())
    }
}


struct Api(Option<fn(&Connection, Context, Response)>);

impl Handler for Api {
    fn handle_request(&self, context: Context, mut response: Response) {
        //Collect the accepted methods from the provided hyperlinks
        let mut methods: Vec<_> = context.hyperlinks.iter().filter_map(|l| l.method.clone()).collect();
        methods.push(context.method.clone());

        //Setup cross origin resource sharing
        response.headers_mut().set(AccessControlAllowOrigin::Any);
        response.headers_mut().set(AccessControlAllowMethods(methods));
        response.headers_mut().set(AccessControlAllowHeaders(vec![UniCase("content-type".into())]));

        //Get the database from the global storage
        let config: &Config = if let Some(config) = context.global.get() {
            config
        } else {
            error!("expected a globally accessible Database");
            response.set_status(StatusCode::InternalServerError);
            return
        };
        let uri = config.lookup_str("connection_uri").expect("Could not find a connection URI");
        let database = Connection::connect(uri, TlsMode::None).unwrap();

        if let Some(action) = self.0 {
            action(&database, context, response);
        }
    }
}
