
use std::collections::HashMap;
use rand::Rng;
use rand;

use rustful::{Context, Response};
use rustful::header::ContentType;
use rustc_serialize::json;
use postgres::Connection;
use rustful::StatusCode;
use uuid::Uuid;

use structs::{Assignment, Participant};

const ALREADY_EXISTS : &'static str = "A set of assignments already exists.";
const DELETE_FAILED : &'static str = "Removing the current Secret Santa assignments failed.";

pub fn get_santa(database: &Connection, context: Context, mut response: Response) {
    let mut res = Vec::new();

    for row in &database.query("SELECT * FROM users", &[]).unwrap() {
        res.push(Assignment {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            code: row.get("code"),
            assignee: row.get("assignee")
        })
    }

    response.headers_mut().set(ContentType::json());
    response.send(json::encode(&res).unwrap());
}

pub fn delete_santa(database: &Connection, context: Context, mut response: Response) {
    or_abort!(
        database.execute("DELETE FROM users", &[]).ok(),
        response,
        StatusCode::InternalServerError,
        DELETE_FAILED
    );

    response.set_status(StatusCode::NoContent);
}

pub fn post_santa(database: &Connection, mut context: Context, mut response: Response) {
    let participants: Vec<Participant> = context.body.decode_json_body().unwrap();

    let count: i64 = database.query("SELECT COUNT(*) FROM users", &[]).unwrap().get(0).get(0);
    abort_if!(
        count != 0,
        response,
        StatusCode::Conflict,
        ALREADY_EXISTS
    );

    let assignments = create_assignments(&participants);

    let stmt = database.prepare("INSERT INTO users (name, email, code, assignee) VALUES ($1, $2, $3, $4)").unwrap();
    for (participant, assignee) in assignments {
        let code = Uuid::new_v4();
        stmt.execute(&[&participant.name, &participant.email, &code, &assignee.name]).unwrap();
    }

    response.set_status(StatusCode::Created);
}

fn create_assignments(names: &Vec<Participant>) -> HashMap<Participant, Participant> {
    let names1 = names.clone();
    let mut names2: Vec<Participant> = names1.to_vec();
    rand::thread_rng().shuffle(&mut *names2);
    let names3 = names2.clone();

    let mut assignments = HashMap::new();
    let mut giver = None;

    for receiver in names2 {
        if giver.is_some() {
            assignments.insert(giver.unwrap(), receiver.clone());
        }
        giver = Some(receiver);
    }
    let ref receiver = names3[0];
    assignments.insert(giver.unwrap(), receiver.clone());

    assignments
}