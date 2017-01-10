use std::collections::HashMap;
use std::hash::Hash;
use rand::Rng;
use rand;

use rustful::{Context, Response};
use rustful::header::ContentType;
use rustc_serialize::json;

use postgres::Connection;
use rustful::StatusCode;

macro_rules! or_abort {
    ($e: expr, $response: expr, $error_message: expr) => (
        if let Some(v) = $e {
            v
        } else {
            $response.set_status(StatusCode::BadRequest);
            $response.headers_mut().set(ContentType(content_type!(Text / Plain; Charset = Utf8)));
            $response.send($error_message);
            return
        }
    )
}

#[derive(RustcDecodable, RustcEncodable)]
struct Assignment {
    id: i32,
    name: String,
    email: String,
    code: String,
    assignee: String
}

#[derive(RustcDecodable, RustcEncodable, Clone, Hash, PartialEq, Eq, Debug)]
struct Participant {
    name: String,
    email: String
}

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

pub fn delete_santa(database: &Connection, context: Context, response: Response) {

}

pub fn post_santa(database: &Connection, mut context: Context, mut response: Response) {
    let participants: Vec<Participant> = context.body.decode_json_body().unwrap();

    let count: i64 = database.query("SELECT COUNT(*) FROM users", &[]).unwrap().get(0).get(0);
    if count != 0 {
        response.set_status(StatusCode::Conflict);
        response.headers_mut().set(ContentType(content_type!(Text / Plain; Charset = Utf8)));
        response.send("A set of assignments already exists");
        return
    }

    let assignments = create_assignments(&participants);
    println!("{:?}", assignments);

    let stmt = database.prepare("INSERT INTO users (name, email, code, assignee) VALUES ($1, $2, $3, $4)").unwrap();
    for (participant, assignee) in assignments {
        stmt.execute(&[&participant.name, &participant.email, &String::from("code"), &assignee.name]);
    }
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
    assignments.insert(giver.unwrap(), Participant::from((*receiver).clone()));

    assignments
}