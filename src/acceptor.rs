use compax::{paxos_kv, BallotID};
use paxos_kv::paxos_service_server::{PaxosService, PaxosServiceServer};
use paxos_kv::{PhaseStatus, ProposeReply, ProposeRequest};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct PaxosAcceptor {}

#[tonic::async_trait]
impl PaxosService for PaxosAcceptor {
    async fn propose(
        &self,
        request: Request<ProposeRequest>,
    ) -> Result<Response<ProposeReply>, Status> {
        println!("Got a call from {:?}", request);
        let reply = ProposeReply {
            status: PhaseStatus::Reject.into(),
            accepted_proposal: None.into(),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = "[::1]:50051".parse()?;
    let acceptor: PaxosAcceptor = PaxosAcceptor::default();

    Server::builder()
        .add_service(PaxosServiceServer::new(acceptor))
        .serve(address)
        .await?;

    Ok(())
}
