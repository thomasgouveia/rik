use route_recognizer;

#[allow(dead_code)]
pub fn get(_: &mut tiny_http::Request, _: &route_recognizer::Params) {
    println!("Get all instance");
}

#[allow(dead_code)]
pub fn create(_: &mut tiny_http::Request, _: &route_recognizer::Params) {
    println!("Create an instance");
}

#[allow(dead_code)]
pub fn delete() {}
