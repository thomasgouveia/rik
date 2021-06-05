pub mod common {
    tonic::include_proto!("common");
}

pub mod controller {
    tonic::include_proto!("controller");
}

pub mod worker {
    tonic::include_proto!("worker");
}

fn main() {
    println!("Hello, world!");
}
