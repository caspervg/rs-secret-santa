use postgres::Connection;

use rustful::{Context, Response};
use rustful::header::ContentType;
use rustc_serialize::json;
use rustful::StatusCode;
use uuid::Uuid;

use structs::{Assignment};

const INVALID_CODE   : &'static str = "You entered am invalid code.";
const INCORRECT_CODE : &'static str = "You entered an incorrect code.";

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

    response.headers_mut().set(ContentType::json());
    response.send(json::encode(&user).unwrap());
}
