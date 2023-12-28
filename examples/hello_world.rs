use cgi::text_response;

#[cgi::main]
fn main(_request: cgi::Request) -> cgi::Response {
    text_response(200, "Hello World!")
}
