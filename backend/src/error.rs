use rocket::Responder;

#[derive(Responder)]
pub enum Error {
    WrongGameId(String),
}
