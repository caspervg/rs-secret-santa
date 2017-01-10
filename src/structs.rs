use uuid::Uuid;

macro_rules! or_abort {
    ($e: expr, $response: expr, $status: expr, $error_message: expr) => (
        if let Some(v) = $e {
            v
        } else {
            $response.set_status($status);
            $response.headers_mut().set(ContentType(content_type!(Text / Plain; Charset = Utf8)));
            $response.send($error_message);
            return
        }
    )
}

macro_rules! abort_if {
    ($e: expr, $response: expr, $status: expr, $error_message: expr) => (
        if $e {
            $response.set_status($status);
            $response.headers_mut().set(ContentType(content_type!(Text / Plain; Charset = Utf8)));
            $response.send($error_message);
            return
        } else {
        }
    )
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Assignment {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub code: Uuid,
    pub assignee: String
}

#[derive(RustcDecodable, RustcEncodable, Clone, Hash, PartialEq, Eq, Debug)]
pub struct Participant {
    pub name: String,
    pub email: String
}