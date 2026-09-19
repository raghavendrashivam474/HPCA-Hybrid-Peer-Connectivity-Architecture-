use hpca::core::{CandidateAddress, PeerId};
use hpca::protocol::{ProbeRequest, ProbeResponse, ProbeStateMachine};
use hpca::session::Session;
use hpca::transport::{LoopbackTransport, Transport};
use std::thread;

#[test]
fn test_phase1_e2e_integration_pipeline() {
    // 1. Peer Identities (S1.1)
    let peer_alice = PeerId::random();
    let peer_bob = PeerId::random();

    // 2. Candidate Addresses (S1.2)
    let address_alice = CandidateAddress::Memory(5001);
    let address_bob = CandidateAddress::Memory(5002);

    // Initializing loopback drivers for Alice & Bob
    let mut transport_alice = LoopbackTransport::new();
    transport_alice.listen(address_alice).unwrap();

    let mut transport_bob = LoopbackTransport::new();
    transport_bob.listen(address_bob).unwrap();

    // 3. Reachability Probing Pipeline (S1.3) + Loopback Transport (S1.4) + Session (S1.4)
    let bob_handle = thread::spawn(move || {
        // Bob awaits Alice's reachability probe
        let conn_from_alice = transport_bob.accept().unwrap();

        // Read raw probe request
        let raw_req = conn_from_alice.recv().unwrap();
        let probe_req = ProbeRequest::from_bytes(&raw_req).unwrap();

        // Assert probe was intended for Bob
        assert_eq!(probe_req.target_peer_id, peer_bob);

        // Formulate and return ProbeResponse
        let probe_resp = ProbeResponse::from_request(&probe_req, peer_bob);
        conn_from_alice.send(&probe_resp.to_bytes()).unwrap();

        // Probe complete. Bob uplevels connection to a secure Session
        let session_bob = Session::bind(peer_bob, peer_alice, conn_from_alice);

        // Exchange secure context session frames
        let hello_alice = session_bob.recv_frame().unwrap();
        assert_eq!(
            hello_alice,
            b"Hello Bob, this is Alice inside the Session context."
        );
        session_bob
            .send_frame(b"Acknowledged, Alice. Session established!")
            .unwrap();
    });

    // Alice dials Bob to perform the Reachability Probe
    let mut probe_sm = ProbeStateMachine::new(peer_bob);
    let conn_to_bob = transport_alice.dial(peer_bob, address_bob).unwrap();

    // Send probe request over the transport channel
    let raw_req = probe_sm.on_sent().unwrap();
    conn_to_bob.send(&raw_req).unwrap();

    // Read and verify probe response
    let raw_resp = conn_to_bob.recv().unwrap();
    probe_sm.handle_response(&raw_resp).unwrap();

    // Verification check: Reachability Engine asserts Succeeded
    assert!(matches!(
        probe_sm.state(),
        hpca::protocol::ProbeState::Succeeded(_)
    ));

    // Reachability confirmed. Alice uplevels physical link to logical Session
    let session_alice = Session::bind(peer_alice, peer_bob, conn_to_bob);
    assert!(session_alice.session_id() > 0);

    // Send session application frames
    session_alice
        .send_frame(b"Hello Bob, this is Alice inside the Session context.")
        .unwrap();
    let resp = session_alice.recv_frame().unwrap();
    assert_eq!(resp, b"Acknowledged, Alice. Session established!");

    bob_handle.join().unwrap();
}
