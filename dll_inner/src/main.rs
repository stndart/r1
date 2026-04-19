use tonic::{Request, Response, Status, transport::Server};

use one::one_server::{One, OneServer};
use one::{AddReply, AddRequest};

pub mod one {
    tonic::include_proto!("one");
}

#[link(name = "example.dll", kind = "dylib")]
unsafe extern "C" {
    fn add(left: i64, right: i64) -> i64;
}

fn add_impl(a: i64, b: i64) -> i64 {
    unsafe { add(a, b) }
}

#[derive(Debug, Default)]
pub struct MyOne {}

#[tonic::async_trait]
impl One for MyOne {
    async fn add(&self, request: Request<AddRequest>) -> Result<Response<AddReply>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();
        let reply = AddReply {
            result: add_impl(request.a, request.b),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let one_server = MyOne::default();

    Server::builder()
        .add_service(OneServer::new(one_server))
        .serve(addr)
        .await?;

    Ok(())
}
