use route_recognizer;
use std::io;

mod instance;
mod tenant;
mod workload;

type Handler = fn(&mut tiny_http::Request, &route_recognizer::Params);

pub struct Router {
    routes: Vec<(tiny_http::Method, route_recognizer::Router<Handler>)>,
}

impl Router {
    pub fn new() -> Router {
        let mut get = route_recognizer::Router::<Handler>::new();
        let mut post = route_recognizer::Router::<Handler>::new();

        get.add("/instance", instance::get);
        post.add("/instance", instance::create);

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
        println!("Handle, {}", request.method());
        self.routes
            .iter()
            .find(|&&(ref method, _)| method == request.method())
            .and_then(|&(_, ref routes)| {
                if let Ok(res) = routes.recognize(request.url()) {
                    res.handler();
                    Some(tiny_http::Response::new_empty(tiny_http::StatusCode::from(
                        201,
                    )))
                } else {
                    None
                }
            })
    }
}
