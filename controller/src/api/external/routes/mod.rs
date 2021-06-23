use route_recognizer;
use std::io;

use crate::api;

mod instance;
mod tenant;
mod workload;

type Handler = fn(
    &mut tiny_http::Request,
    &route_recognizer::Params,
) -> Result<tiny_http::Response<io::Empty>, api::RikError>;

pub struct Router {
    routes: Vec<(tiny_http::Method, route_recognizer::Router<Handler>)>,
}

impl Router {
    pub fn new() -> Router {
        let mut get = route_recognizer::Router::<Handler>::new();
        let mut post = route_recognizer::Router::<Handler>::new();

        get.add("/instance", instance::get);
        post.add("/instance", instance::create as Handler);

        Router {
            routes: vec![
                ("GET".parse().unwrap(), get),
                ("POST".parse().unwrap(), post),
            ],
        }
    }

    pub fn handle(
        &self,
        request: &mut tiny_http::Request,
    ) -> Option<tiny_http::Response<io::Empty>> {
        self.routes
            .iter()
            .find(|&&(ref method, _)| method == request.method())
            .and_then(|&(_, ref routes)| {
                if let Ok(res) = routes.recognize(request.url()) {
                    Some(res.handler()(request, &res.params()).unwrap())
                } else {
                    None
                }
            })
    }
}
