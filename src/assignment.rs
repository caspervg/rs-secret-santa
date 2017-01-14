use postgres::Connection;

use rustful::{Context, Response};
use rustful::header::ContentType;
use rustful::StatusCode;
use uuid::Uuid;
use tera::{Tera, Context as TeraContext};
use chrono::{Local, Datelike};

use structs::{Assignment};

const INCORRECT_CODE : &'static str = "You entered an incorrect code.";

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = compile_templates!("src/templates/**/*");
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

pub fn get_assignment(database: &Connection, context: Context, mut response: Response) {
    let code = context.variables.get("code").unwrap().into_owned();

    let uuid = Uuid::parse_str(&code).unwrap_or(Uuid::new_v4());
    let count: i64 = database.query(
        "SELECT COUNT(*) FROM users WHERE code = $1",
        &[&uuid]
    ).unwrap().get(0).get(0);
    abort_if!(
        count == 0,
        response,
        StatusCode::NotFound,
        INCORRECT_CODE
    );

    let qry = database.query(
        "SELECT * FROM users WHERE code = $1",
        &[&uuid]
    ).unwrap();
    let row = qry.get(0);

    let user = Assignment {
        id: row.get("id"),
        name: row.get("name"),
        email: row.get("email"),
        code: row.get("code"),
        assignee: row.get("assignee")
    };
    let dt = Local::now();

    let mut tera_ctx = TeraContext::new();
    tera_ctx.add("name", &user.name);
    tera_ctx.add("assignee", &user.assignee);
    tera_ctx.add("year", &dt.year());

    match TEMPLATES.render("assignment/assignment.html", tera_ctx) {
        Ok(s) => {
            response.headers_mut().set(ContentType::html());
            response.send(s)
        },
        Err(e) => {
            response.set_status(StatusCode::InternalServerError);
            response.send("An error occurred while rendering the assignment template.");
            println!("Error: {}", e);
            for e in e.iter().skip(1) {
                println!("Reason: {}", e);
            }
        }
    };
}
