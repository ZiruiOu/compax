use compax::{paxos_kv, BallotID};
use paxos_kv::paxos_service_server::{PaxosService, PaxosServiceServer};
use paxos_kv::{AcceptReply, AcceptRequest, PhaseStatus, Proposal, ProposeReply, ProposeRequest};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct PaxosAcceptorInner {
    accepted_number: BallotID,
    accepted_value: Option<u64>,
    highest_number: BallotID,
}

impl PaxosAcceptorInner {
    pub fn new() -> Self {
        PaxosAcceptorInner {
            accepted_number: Default::default(),
            accepted_value: None,
            highest_number: Default::default(),
        }
    }

    pub fn on_propose(&mut self, proposal: Proposal) -> ProposeReply {
        let propose_number = proposal.propose_id.unwrap().into();
        if self.accepted_value.is_none() || self.highest_number < propose_number {
            self.highest_number = propose_number;
            ProposeReply {
                status: PhaseStatus::Accept.into(),
                accepted_proposal: Some(Proposal {
                    propose_id: Some(self.accepted_number.clone().into()),
                    propose_value: self.accepted_value.clone(),
                }),
            }
        } else {
            ProposeReply {
                status: PhaseStatus::Reject.into(),
                accepted_proposal: None,
            }
        }
    }

    pub fn on_accept(&mut self, proposal: Proposal) -> AcceptReply {
        let propose_number = proposal.propose_id.unwrap().into();
        let propose_value = proposal.propose_value.unwrap();
        if self.accepted_value.is_none() || self.highest_number <= propose_number {
            self.accepted_number = propose_number.clone();
            self.accepted_value = Some(propose_value);
            self.highest_number = propose_number.clone();
            AcceptReply {
                status: PhaseStatus::Accept.into(),
            }
        } else {
            AcceptReply {
                status: PhaseStatus::Reject.into(),
            }
        }
    }
}

#[derive(Debug)]
pub struct PaxosAcceptor {
    pub inner: Arc<Mutex<PaxosAcceptorInner>>,
}

impl PaxosAcceptor {
    pub fn new() -> Self {
        PaxosAcceptor {
            inner: Arc::new(Mutex::new(PaxosAcceptorInner::new())),
        }
    }
}

#[tonic::async_trait]
impl PaxosService for PaxosAcceptor {
    async fn propose(
        &self,
        request: Request<ProposeRequest>,
    ) -> Result<Response<ProposeReply>, Status> {
        println!("propose: Got a rpc from {:?}", request);
        let proposal = request.into_inner().proposal.unwrap();
        let reply: ProposeReply;
        {
            let mut inner = self.inner.lock().unwrap();
            reply = inner.on_propose(proposal);
        }
        Ok(Response::new(reply))
    }

    async fn accept(
        &self,
        request: Request<AcceptRequest>,
    ) -> Result<Response<AcceptReply>, Status> {
        println!("accept : Got a rpc from {:?}", request);
        let inner_proposal = request.into_inner().proposal.unwrap();
        let reply = self.inner.lock().unwrap().on_accept(inner_proposal);
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = "[::1]:50051".parse()?;
    let acceptor: PaxosAcceptor = PaxosAcceptor::new();

    Server::builder()
        .add_service(PaxosServiceServer::new(acceptor))
        .serve(address)
        .await?;

    Ok(())
}
