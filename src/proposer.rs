use compax::{paxos_kv, BallotID, PaxosProposalStatus};
use paxos_kv::paxos_service_client::PaxosServiceClient;
use paxos_kv::{AcceptReply, AcceptRequest, PhaseStatus, Proposal, ProposeReply, ProposeRequest};
use tonic::Request;

pub struct PaxosProposer {
    propose_number: u64,
    proposer_id: u64,
    acceptor_addrs: Vec<&'static str>,
}

impl PaxosProposer {
    pub fn new(proposer_id: u64, acceptor_addrs: Vec<&'static str>) -> Self {
        PaxosProposer {
            propose_number: 0,
            proposer_id: proposer_id,
            acceptor_addrs: acceptor_addrs,
        }
    }

    pub async fn propose(
        &mut self,
        value: u64,
    ) -> Result<PaxosProposalStatus, Box<dyn std::error::Error>> {
        // Phase-1 : propose
        self.propose_number += 1;
        let ballot_id = BallotID::new(self.propose_number, self.proposer_id);
        let mut proposal = Proposal {
            propose_id: Some(ballot_id.into()),
            propose_value: value.into(),
        };
        let mut results = self.send_all_propose(proposal.clone()).await?;

        // If we receive accepted from the quorom, we can step to phase-2
        let num_accepted = results
            .iter()
            .filter(|&x| x.status() == PhaseStatus::Accept)
            .count();
        let quorom_size = self.acceptor_addrs.len() / 2 + 1;

        if num_accepted > quorom_size {
            // Choose the history value for the current round proposal
            let mut propose_id: BallotID = Default::default();
            let mut propose_value: Option<u64> = None;
            for reply in results.iter_mut() {
                let p = reply.accepted_proposal.as_ref().unwrap();
                let pid = p.propose_id.as_ref().unwrap().clone().into();
                if propose_value.is_none() || propose_id < pid {
                    propose_id = pid;
                    propose_value = p.propose_value;
                }
            }

            if !propose_value.is_none() {
                proposal.propose_value = propose_value;
            }

            let results = self.send_all_accept(proposal).await?;
            let num_accepted = results
                .iter()
                .filter(|&x| x.status() == PhaseStatus::Accept)
                .count();

            if num_accepted > quorom_size {
                return Ok(PaxosProposalStatus::Ok);
            }
        }

        Ok(PaxosProposalStatus::Fail)
    }

    pub async fn send_all_propose(
        &mut self,
        proposal: Proposal,
    ) -> Result<Vec<ProposeReply>, Box<dyn std::error::Error>> {
        let mut reply_list: Vec<ProposeReply> = Vec::new();
        for (_, addr) in self.acceptor_addrs.iter_mut().enumerate() {
            let mut client_stub = PaxosServiceClient::connect(*addr).await?;
            let request = Request::new(ProposeRequest {
                proposal: proposal.clone().into(),
            });
            let reply = client_stub.propose(request).await?;
            reply_list.push(reply.into_inner());
        }
        Ok(reply_list)
    }

    pub async fn send_all_accept(
        &mut self,
        proposal: Proposal,
    ) -> Result<Vec<AcceptReply>, Box<dyn std::error::Error>> {
        let mut reply_list: Vec<AcceptReply> = Vec::new();
        for (_, addr) in self.acceptor_addrs.iter().enumerate() {
            let mut client_stub = PaxosServiceClient::connect(*addr).await?;
            let request = Request::new(AcceptRequest {
                proposal: proposal.clone().into(),
            });
            let reply = client_stub.accept(request).await?;
            reply_list.push(reply.into_inner());
        }
        Ok(reply_list)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "http://[::1]:50051";
    let mut client_stub = PaxosServiceClient::connect(address).await?;

    let ballot_id = BallotID::new(100, 2);

    let proposal = Proposal {
        propose_id: Some(ballot_id.into()),
        propose_value: Some(114514),
    };

    // Phase-1 : Propose
    let request = Request::new(ProposeRequest {
        proposal: proposal.clone().into(),
    });
    let reply = client_stub.propose(request).await?;

    println!("Phase-1: receive reply: {:?}", reply);

    // Phase-2: Accept
    if PhaseStatus::Accept == reply.into_inner().status() {
        let request = Request::new(AcceptRequest {
            proposal: proposal.clone().into(),
        });
        let reply = client_stub.accept(request).await?;

        println!("Receive reply: {:?}", reply);
    }
    Ok(())
}
