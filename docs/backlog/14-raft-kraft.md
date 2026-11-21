# RAFT: KRaft consensus

> Crates: `alluvium-raft` (layer 1) · Kafka counterpart: `raft/src/main/java/org/apache/kafka/raft` · Milestones: M2-M3

The KRaft metadata quorum as a sans-I/O state machine: elections with pre-vote, replication via Fetch, snapshots, dynamic voters, observers, persistence and its driver.

## [To Do] RAFT-F01 Raft state machine core
P0 · M2 · Feature · deps: TYPE-F06, TYPE-F07
Scope: States (unattached, prospective, candidate, leader, follower, resigned), epochs, single-voter self-election.

### [To Do] RAFT-0001 Spike: sans-I/O KRaft client design (inputs, actions, poll deadline, deferred responses)
P0 · M2 · Spike · deps: none · labels: docs · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java poll @4.4.0; ADR-0002
Done when: ADR 'KRaft sans-I/O client' is accepted in docs/adr/; it defines the Input enum (inbound request, inbound response, tick(now), append, register listener, resign, shutdown), the Action enum (send request/response, persist election state, append to log, flush, fire listener, complete deferred response), how KafkaRaftClient.poll's returned timeout becomes a next-deadline output, and how the fetch purgatory maps to deferred responses.

### [To Do] RAFT-0002 Raft value types: ReplicaKey, OffsetAndEpoch, LeaderAndEpoch, Endpoints, ElectionState
P0 · M2 · Task · deps: RAFT-0001 · ref: raft/src/main/java/org/apache/kafka/raft/ReplicaKey.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/ElectionState.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/Endpoints.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/LeaderAndEpoch.java @4.4.0
Done when: Types exist in alluvium-raft with Debug; OffsetAndEpoch orders by epoch then offset (property test against a reference comparator); ReplicaKey matches when ids are equal and either directory id is absent or both are equal; Endpoints is keyed by listener name; ElectionState carries epoch, optional leader id, optional voted key and voter ids.

### [To Do] RAFT-0003 VoterSet: voter nodes with listeners and supported kraft.version, add/remove/update operations
P0 · M2 · Task · deps: RAFT-0002 · ref: raft/src/main/java/org/apache/kafka/raft/VoterSet.java @4.4.0
Done when: VoterSet supports isVoter (id plus optional directory id), isOnlyVoter, voterIds, voter node lookup per listener, addVoter/removeVoter/updateVoter returning None on conflict, voterNodeNeedsUpdate, supportsVersion, and conversion to and from VotersRecord; unit tests cover each operation including voters without directory ids.

### [To Do] RAFT-0004 EpochElection: per-voter vote tracking with majority granted/rejected decisions
P0 · M2 · Task · deps: RAFT-0003 · ref: raft/src/main/java/org/apache/kafka/raft/internals/EpochElection.java @4.4.0
Done when: EpochElection records granted/rejected/unrecorded per voter; majority is voters/2+1; isVoteGranted and isVoteRejected (granted plus unrecorded below majority) are correct for 1 to 7 voters in a property test; recording a vote from a non-voter is an invariant violation; unrecordedVoters lists only voters still to ask.

### [To Do] RAFT-0005 raft::QuorumConfig plain struct with Kafka defaults and invariants
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java @4.4.0
Done when: QuorumConfig holds election timeout (1000 ms), fetch timeout (2000 ms), election backoff max (1000 ms), append linger (25 ms), request timeout (2000 ms), retry backoff (20 ms), fetch max bytes and fetch snapshot max bytes (1 MiB) and auto-join (false); construction rejects non-positive timeouts with a typed error; the mapping from controller.quorum.* keys stays in CFG-F10.

### [To Do] RAFT-0006 Deterministic election and fetch timers with seeded election-timeout jitter
P0 · M2 · Task · deps: RAFT-0005 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java randomElectionTimeoutMs @4.4.0
Done when: A timer type driven only by the injected `now` exposes remaining/expired/reset; the randomized election timeout is election.timeout.ms plus a value in [0, election.timeout.ms) drawn from the injected RNG; a seeded test reproduces the same sequence and a property test bounds every value.

### [To Do] RAFT-0007 EpochState variants: Unattached, Prospective, Candidate, Leader, Follower, Resigned
P0 · M2 · Task · deps: RAFT-0002, RAFT-0004, RAFT-0006 · ref: raft/src/main/java/org/apache/kafka/raft/EpochState.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/UnattachedState.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/ProspectiveState.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/CandidateState.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/FollowerState.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/ResignedState.java @4.4.0
Done when: One enum with a struct per state carrying exactly Kafka's fields (epoch, leader id/endpoints, voted key, epoch election, high watermark, election or fetch timer, preferred successors and unacked voters for Resigned, fetching snapshot and hasFetchedFromLeader for Follower); election() and highWatermark() accessors match each Kafka state class; unit tests construct each state.

### [To Do] RAFT-0008 Transitions to Unattached, Follower and Resigned with Kafka's validity rules
P0 · M2 · Task · deps: RAFT-0007 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java transitionToUnattached, transitionToFollower, transitionToResigned @4.4.0
Done when: transition_to_unattached requires a higher epoch (or the same epoch from Prospective, keeping the voted key) and inherits the remaining election timeout from Candidate/Unattached/Prospective/Resigned (infinite for observers); transition_to_follower rejects empty endpoints, the local id, a lower epoch and same-epoch moves unless more endpoints are learned; transition_to_resigned is only valid from Leader; a table-driven test covers every from/to pair.

### [To Do] RAFT-0009 Transitions to Prospective, Candidate and Leader plus voted-state additions
P0 · M2 · Task · deps: RAFT-0007 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java transitionToProspective, transitionToCandidate, transitionToLeader, unattachedAddVotedState @4.4.0
Done when: Prospective is reachable from any non-leader, non-prospective voter state without an epoch change; Candidate only from Prospective with epoch+1 and a self vote; Leader only from Candidate with a granted majority; unattached_add_voted_state/prospective_add_voted_state reject the local id, other epochs and already-voted states; observers cannot enter Prospective/Candidate/Leader; table-driven tests cover every rule.

### [To Do] RAFT-0010 Durable versus in-memory transitions emit a persist-election-state action first
P0 · M2 · Task · deps: RAFT-0008, RAFT-0009 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java durableTransitionTo, memoryTransitionTo @4.4.0
Done when: Every durable transition (unattached, voted, follower, candidate, initial) returns a PersistElectionState action ordered before any send action; Prospective and Resigned transitions are memory-only; tests assert the action list for each transition matches Kafka's durableTransitionTo/memoryTransitionTo split.

### [To Do] RAFT-0011 Initialize quorum state from the persisted election state and the log end offset and epoch
P0 · M2 · Task · deps: RAFT-0010 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java initialize @4.4.0
Done when: Startup yields: Unattached at the log epoch when the stored epoch is older; Resigned in the same epoch if the node was leader; Candidate if it voted for itself; Follower if the leader's endpoints are known, else Unattached keeping the leader id; Unattached otherwise; a voted state without a local id is a typed error; one test per branch, ported from QuorumStateTest.

### [To Do] RAFT-0012 Leader replica tracking and high watermark from the majority of voter end offsets
P0 · M2 · Task · deps: RAFT-0007 · ref: raft/src/main/java/org/apache/kafka/raft/LeaderState.java maybeUpdateHighWatermark, updateLocalState, updateReplicaState @4.4.0
Done when: LeaderState keeps voter and observer ReplicaState; the high watermark is the end offset at index voters/2 of voters sorted by end offset descending, only once it exceeds the epoch start offset; it never decreases except when voter membership changes (logged); a non-monotonic local end offset is an invariant violation; property test over random fetch sequences.

### [To Do] RAFT-0013 Single-voter self-election at startup without network round trips
P0 · M2 · Task · deps: RAFT-0011, RAFT-0012 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java initialize, onBecomeLeader @4.4.0
Done when: When the local node is the only voter and starts Unattached, Follower or Resigned it moves Prospective -> Candidate (epoch+1, persisted) -> Leader in one step, setting the epoch start offset to the log end offset; a combined-mode test with one voter reaches Leader with the expected epoch on first and repeated restarts.

### [To Do] RAFT-0014 Transition records with typed reasons returned by every state change
P0 · M2 · Task · deps: RAFT-0008, RAFT-0009 · labels: observability
Done when: Every transition returns a StateTransition {from, to, epoch, leader_id, reason} where reason is an enum (startup, election_timeout, fetch_timeout, vote_majority, higher_epoch_seen, leader_discovered, check_quorum_expired, resign_requested, shutdown, election_lost) with stable snake_case Display; unit tests assert the reason for each transition path.

### [To Do] RAFT-0015 RaftError enum and its single exhaustive Kafka error-code mapping
P0 · M2 · Task · deps: RAFT-0002, ERR-F02 · ref: raft/src/main/java/org/apache/kafka/raft/errors/NotLeaderException.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/errors/BufferAllocationException.java @4.4.0
Done when: alluvium-raft defines a thiserror RaftError whose variants carry epoch/leader/offset context (not leader, stale epoch, invalid offset, buffer full, shutting down); one exhaustive match maps it to ErrorCode with no `_` arm; tests cover each variant.

### [To Do] RAFT-0016 Property test: arbitrary timer and higher-epoch input sequences keep quorum invariants
P0 · M2 · Task · deps: RAFT-0011, RAFT-0013 · labels: test
Done when: A proptest drives the state machine with random ticks, higher-epoch notices and restarts (re-initializing from the persisted state) and asserts epochs never decrease, at most one vote is persisted per epoch, leaders only come from candidates with a majority and observers never vote or campaign; failures print a replayable seed.

### [To Do] RAFT-0017 RequestManager: per-destination in-flight tracking, request timeout, retry backoff and bootstrap rotation
P0 · M3 · Task · deps: RAFT-0005 · ref: raft/src/main/java/org/apache/kafka/raft/RequestManager.java @4.4.0
Done when: Each destination is ready, awaiting a response (by correlation id) or backing off; responses with unexpected correlation ids are discarded; a request times out after controller.quorum.request.timeout.ms; failures back off controller.quorum.retry.backoff.ms; bootstrap servers are tried round-robin skipping backing-off ones; reset_all clears state on epoch change; unit tests for each rule.

### [To Do] RAFT-0018 Shared request validation: voter-only, leader-only, partition and voter-key checks
P0 · M3 · Task · deps: RAFT-0007, RAFT-0015 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java validateVoterOnlyRequest, validateLeaderOnlyRequest, isValidVoterKey @4.4.0
Done when: validate_voter_only returns FENCED_LEADER_EPOCH for an older epoch and INVALID_REQUEST for a negative remote id; validate_leader_only returns FENCED_LEADER_EPOCH, UNKNOWN_LEADER_EPOCH, NOT_LEADER_OR_FOLLOWER or BROKER_NOT_AVAILABLE (shutting down) in Kafka's order; requests not for __cluster_metadata-0 (or its topic id) get INVALID_REQUEST; voter-key mismatch is detected with directory-id rules; unit test per error.

### [To Do] RAFT-0019 Common response handling and maybeTransition on learned leader and epoch
P0 · M3 · Task · deps: RAFT-0008, RAFT-0017 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java maybeHandleCommonResponse, maybeTransition, hasConsistentLeader @4.4.0
Done when: Responses with an older epoch or UNKNOWN_LEADER_EPOCH are ignored; a higher epoch, FENCED_LEADER_EPOCH or NOT_LEADER_OR_FOLLOWER triggers maybe_transition (Follower when the leader is known, else Unattached); a same-epoch response naming an unknown leader makes the node a follower; BROKER_NOT_AVAILABLE backs off; INVALID_VOTER_KEY is logged; an inconsistent leader for the same epoch or INVALID_REQUEST is an invariant violation; leader endpoints fall back to the voter set; tests per branch.

### [To Do] RAFT-0020 Top-level response errors: backoff, fatal cluster authorization failure
P0 · M3 · Task · deps: RAFT-0019 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleTopLevelError, handleUnexpectedError @4.4.0; raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientClusterAuthTest.java @4.4.0
Done when: A top-level BROKER_NOT_AVAILABLE backs the destination off; CLUSTER_AUTHORIZATION_FAILED is fatal for the raft client with a clear error naming the peer; any other unexpected error is logged once and backs off; cases from KafkaRaftClientClusterAuthTest at 4.4.0 pass.

### [To Do] RAFT-0021 Crate docs: KRaft state diagram, transition table and glossary terms
P1 · M2 · Task · deps: RAFT-0008, RAFT-0009 · labels: docs
Done when: alluvium-raft's lib.rs `//!` documents responsibility, layer, Kafka counterpart, the six states with an ASCII transition diagram and invariants; docs/glossary.md gains prospective, resigned, observer, voter set and epoch start offset.

## [To Do] RAFT-F02 Vote and pre-vote
P0 · M3 · Feature · deps: RAFT-F01
Scope: Vote v0-v2 handling, pre-vote (KIP-996), election timeouts with jitter.

### [To Do] RAFT-0022 Vote request validation: cluster id, partition, illegal epochs and negative offsets
P0 · M3 · Task · deps: RAFT-0018, RAFT-F15 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleVoteRequest @4.4.0
Done when: A Vote request with a mismatched cluster id gets INCONSISTENT_CLUSTER_ID; a wrong partition gets INVALID_REQUEST; last offset < 0, last epoch < 0, or last epoch >= candidate epoch (> for pre-vote) gets INVALID_REQUEST; an older replica epoch gets FENCED_LEADER_EPOCH; a higher epoch moves the receiver to Unattached before deciding; unit test per case.

### [To Do] RAFT-0023 Vote granting rules per state with the log up-to-date check
P0 · M3 · Task · deps: RAFT-0022 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java unattachedOrProspectiveCanGrantVote @4.4.0; raft/src/main/java/org/apache/kafka/raft/FollowerState.java canGrantVote @4.4.0; raft/src/main/java/org/apache/kafka/raft/CandidateState.java canGrantVote @4.4.0
Done when: The candidate's (last offset, last epoch) must compare >= the local end offset and epoch; Unattached/Prospective grant a standard vote only if not already voted for another key and no leader is known; Leader always rejects; Follower, Candidate and Resigned reject standard votes; a table test covers all six states for vote and pre-vote, ported from KafkaRaftClientTest/QuorumStateTest.

### [To Do] RAFT-0024 Pre-vote (KIP-996): granting without recording a vote or changing the epoch
P0 · M3 · Task · deps: RAFT-0023 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleVoteRequest @4.4.0; KIP-996
Done when: A pre-vote is granted when the prospective's log is up to date, except a Follower that has already fetched from its leader rejects it; granting a pre-vote never persists a voted key or bumps the epoch; the replica epoch rule allows last epoch == replica epoch for pre-votes; tests show a partitioned node rejoining cannot disrupt a healthy leader.

### [To Do] RAFT-0025 Persist the granted vote before responding to a standard Vote request
P0 · M3 · Task · deps: RAFT-0023, RAFT-0010 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java unattachedAddVotedState, prospectiveAddVotedState @4.4.0
Done when: Granting a standard vote in Unattached or Prospective-not-voted transitions to the voted state in the same epoch (persisted, election timer reset) and the response is released only after the persist action; a second candidate in the same epoch is rejected after a restart; tests with a simulated restart between grant and second request.

### [To Do] RAFT-0026 Vote request voter-key check returns INVALID_VOTER_KEY when not addressed to this replica
P0 · M3 · Task · deps: RAFT-0022 · ref: raft/src/main/java/org/apache/kafka/raft/RaftUtil.java voteRequestVoterKey @4.4.0
Done when: When the request names a voter id/directory id that does not match the local replica the response is INVALID_VOTER_KEY with vote not granted, after any epoch transition; Vote v0 requests without voter keys skip the check; unit tests for v0, v1 and v2.

### [To Do] RAFT-0027 Vote response building for v0-v2 with leader id, epoch and node endpoints
P0 · M3 · Task · deps: RAFT-0023 · ref: raft/src/main/java/org/apache/kafka/raft/RaftUtil.java singletonVoteResponse @4.4.0; clients/src/main/resources/common/message/VoteResponse.json @4.4.0
Done when: Responses carry the partition error, current leader id/epoch and vote_granted; v1+ include node endpoints for the leader on the request's listener; v2 carries the pre-vote fields; golden tests compare encoded responses for each version with Kafka's.

### [To Do] RAFT-0028 Vote request building from Prospective (pre-vote) and Candidate states
P0 · M3 · Task · deps: RAFT-0027, RAFT-0017 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java buildVoteRequest, maybeSendVoteRequests @4.4.0
Done when: Requests carry cluster id, candidate epoch (current epoch for pre-vote), candidate id and directory id, voter key of the destination, last offset and last epoch, and pre_vote; they are sent only to unrecorded voters while the election is not lost; tests assert the destinations and fields.

### [To Do] RAFT-0029 Vote response handling: record votes, advance on majority, handle election loss
P0 · M3 · Task · deps: RAFT-0028, RAFT-0019 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleVoteResponse, maybeHandleElectionLoss @4.4.0
Done when: A granted vote in Prospective moves to Candidate on majority and in Candidate to Leader; a rejected majority makes Prospective fall back to Follower (leader known with endpoints) or Unattached, while Candidate waits out its election timeout; responses for a Leader or a non-nominee are ignored; common errors go through the shared handler; tests per path.

### [To Do] RAFT-0030 Prospective receiving UNSUPPORTED_VERSION skips pre-vote and becomes Candidate
P0 · M3 · Task · deps: RAFT-0029 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleVoteResponse @4.4.0
Done when: A top-level UNSUPPORTED_VERSION in a Vote response while Prospective transitions immediately to Candidate (a voter without pre-vote support exists); other top-level errors follow handleTopLevelError; test with a v0-only peer.

### [To Do] RAFT-0031 Election timeouts: Unattached -> Prospective, Prospective timeout, Candidate -> Prospective
P0 · M3 · Task · deps: RAFT-0029, RAFT-0006 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java pollUnattachedAsVoter, pollProspective, pollCandidate @4.4.0
Done when: An Unattached voter whose randomized election timeout expires becomes Prospective; a Prospective whose timeout expires falls back like an election loss; a Candidate whose timeout expires becomes Prospective again (no immediate re-candidacy); observers never time out into elections; tests drive ticks with a seeded clock.

### [To Do] RAFT-0032 Elections during graceful shutdown keep running until elected, a leader appears or the timer ends
P1 · M3 · Task · deps: RAFT-0031 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java pollCandidate, pollProspective, pollUnattachedAsVoter @4.4.0
Done when: With a pending shutdown, Prospective and Candidate keep sending votes but do not start new elections; Unattached voters stay put; the poll deadline is the minimum of the shutdown remaining time and request backoff; tests for each state.

### [To Do] RAFT-0033 Election start and latency tracking hooks for election-latency metrics
P1 · M3 · Task · deps: RAFT-0029 · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateElectionStartMs, maybeUpdateElectionLatency @4.4.0
Done when: Entering Prospective/Candidate records an election start time and becoming Leader yields an election duration in the transition output; values are exact under the simulated clock; consumed by RAFT-F14.

### [To Do] RAFT-0034 Port Kafka's vote and pre-vote client tests
P0 · M3 · Task · deps: RAFT-0030, RAFT-0031, RAFT-0024, RAFT-0025, RAFT-0026 · labels: test · ref: raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientPreVoteTest.java @4.4.0
Done when: Vote and pre-vote cases from KafkaRaftClientTest and KafkaRaftClientPreVoteTest at 4.4.0 run against the sans-I/O client with `// Ported from:` attribution and pass.

## [To Do] RAFT-F03 Begin/EndQuorumEpoch
P0 · M3 · Feature · deps: RAFT-F01
Scope: Leader announcement and graceful resignation.

### [To Do] RAFT-0035 Becoming leader: accumulator at the log end, epoch start offset and BeginQuorumEpoch to all voters
P0 · M3 · Task · deps: RAFT-F10, RAFT-0017 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java onBecomeLeader @4.4.0
Done when: On election the leader initializes the leader epoch at the log end offset, creates its accumulator, schedules the start-of-epoch control batch, resets connections and sends BeginQuorumEpoch to every other voter; tests assert actions and the epoch cache entry.

### [To Do] RAFT-0036 BeginQuorumEpoch request handling with endpoint discovery and voter-key check
P0 · M3 · Task · deps: RAFT-0018, RAFT-0019, RAFT-F15 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleBeginQuorumEpochRequest @4.4.0
Done when: Requests validate cluster id (INCONSISTENT_CLUSTER_ID), partition (INVALID_REQUEST) and voter-only rules; leader endpoints come from the request or the voter set; the receiver transitions to Follower; a mismatched voter key yields INVALID_VOTER_KEY after the transition; tests per case for v0 and v1.

### [To Do] RAFT-0037 BeginQuorumEpoch responses record acknowledgement; periodic resend to voters not fetching
P0 · M3 · Task · deps: RAFT-0035, RAFT-0036 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleBeginQuorumEpochResponse @4.4.0; raft/src/main/java/org/apache/kafka/raft/LeaderState.java needToSendBeginQuorumRequests @4.4.0
Done when: A successful response marks the voter as having acknowledged the leader; the leader reresends BeginQuorumEpoch to voters whose last fetch is older than fetch.timeout/2 each time that timer expires; responses after losing leadership are ignored; tests with a silent voter.

### [To Do] RAFT-0038 Check quorum: the leader resigns when a majority has not fetched within the timeout
P0 · M3 · Task · deps: RAFT-0035 · ref: raft/src/main/java/org/apache/kafka/raft/LeaderState.java timeUntilCheckQuorumExpires, updateCheckQuorumForFollowingVoter @4.4.0
Done when: The leader tracks voters that fetched (Fetch or FetchSnapshot, matching replica key) and resets the check-quorum timer (1.5 x fetch timeout) once a majority (leader counted if it is a voter) has fetched; on expiry it transitions to Resigned; a single-voter quorum never expires; tests with partitioned followers.

### [To Do] RAFT-0039 Resignation: Resigned state with preferred successors ordered by fetch offset
P0 · M3 · Task · deps: RAFT-0038 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java resign, transitionToResigned @4.4.0; raft/src/main/java/org/apache/kafka/raft/LeaderState.java nonLeaderVotersByDescendingFetchOffset @4.4.0
Done when: Resign (API call with the current epoch, shutdown, or check-quorum expiry) moves Leader to Resigned with non-leader voters sorted by descending fetch offset as preferred successors, fails parked fetches with NOT_LEADER_OR_FOLLOWER and resets connections; resign with a stale epoch is a no-op, a future epoch is an error; tests per trigger.

### [To Do] RAFT-0040 Resigned polling: EndQuorumEpoch to unacked voters, then Unattached at epoch+1
P0 · M3 · Task · deps: RAFT-0039 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java pollResigned, buildEndQuorumEpochRequest @4.4.0
Done when: Resigned sends EndQuorumEpoch (with preferred candidates) to voters that have not acknowledged; after its election timeout it becomes Unattached at epoch+1 so it never follows itself; during shutdown it stays Resigned until the shutdown deadline; tests drive ticks and responses.

### [To Do] RAFT-0041 EndQuorumEpoch request handling: followers of that leader shorten their fetch timeout
P0 · M3 · Task · deps: RAFT-0036 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleEndQuorumEpochRequest, endEpochElectionBackoff, strictExponentialElectionBackoffMs @4.4.0
Done when: Requests validate cluster id and voter-only rules, run maybe_transition, and if the receiver follows the resigning leader it overrides the fetch timeout with a strict exponential backoff by its position in the preferred candidates (0 ms for the first, election.backoff.max.ms when absent); tests for positions 0..n and absence.

### [To Do] RAFT-0042 EndQuorumEpoch responses acknowledge the resignation
P0 · M3 · Task · deps: RAFT-0040 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleEndQuorumEpochResponse @4.4.0; raft/src/main/java/org/apache/kafka/raft/ResignedState.java acknowledgeResignation @4.4.0
Done when: A successful response removes the voter from the Resigned state's unacked set; error responses use the common handler; once all voters acknowledge no more EndQuorumEpoch requests are sent; tests.

### [To Do] RAFT-0043 Graceful shutdown of a leader completes after resignation or the shutdown timeout
P0 · M3 · Task · deps: RAFT-0040 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java shutdown, maybeCompleteShutdown @4.4.0
Done when: shutdown(timeout) on a leader resigns immediately, sends EndQuorumEpoch, and completes when a new epoch is observed or the timeout expires; followers and observers complete immediately; the completion is an action the driver awaits; tests for each role.

### [To Do] RAFT-0044 BeginQuorumEpoch and EndQuorumEpoch request/response construction for v0 and v1
P0 · M3 · Task · deps: RAFT-0036, RAFT-0041 · ref: raft/src/main/java/org/apache/kafka/raft/RaftUtil.java singletonBeginQuorumEpochRequest, singletonEndQuorumEpochRequest @4.4.0
Done when: v1 requests carry voter keys and leader endpoints and responses carry node endpoints; v0 omits them and the receiver falls back to the voter set; golden encoding tests per version.

### [To Do] RAFT-0045 Port Kafka's leader announcement, resignation and check-quorum tests
P0 · M3 · Task · deps: RAFT-0042, RAFT-0043, RAFT-0044, RAFT-0037 · labels: test · ref: raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientTest.java @4.4.0
Done when: BeginQuorumEpoch, EndQuorumEpoch, resign and check-quorum cases from KafkaRaftClientTest at 4.4.0 pass against the sans-I/O client with attribution.

## [To Do] RAFT-F04 Log replication via Fetch
P0 · M3 · Feature · deps: RAFT-F01, RAFT-F05
Scope: Controller-listener Fetch, diverging epochs, high watermark, fetch timeouts.

### [To Do] RAFT-0046 Leader Fetch request validation for the metadata partition
P0 · M3 · Task · deps: RAFT-0018, RAFT-F15 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchRequest @4.4.0
Done when: Fetch requests with a mismatched cluster id get INCONSISTENT_CLUSTER_ID, a wrong topic id/partition gets INVALID_REQUEST, and max_wait_ms < 0, fetch offset < 0, last fetched epoch < 0 or > current leader epoch get INVALID_REQUEST; leader-only validation follows; tests per case.

### [To Do] RAFT-0047 Leader fetch completion: diverging epoch, snapshot redirect or records up to max bytes
P0 · M3 · Task · deps: RAFT-0046, RAFT-0012 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java tryCompleteFetchRequest @4.4.0
Done when: An empty follower (fetch offset 0) is told to fetch the latest non-bootstrap snapshot; otherwise validate_offset_and_epoch returns a diverging epoch, a snapshot id, or a valid read of uncommitted records up to the request's max bytes; a valid read updates the replica state and possibly the high watermark; unexpected errors return UNKNOWN_SERVER_ERROR; tests per outcome.

### [To Do] RAFT-0048 Respond immediately or park the fetch: Kafka's six conditions
P0 · M3 · Task · deps: RAFT-0047 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchRequest @4.4.0
Done when: A fetch is answered at once if it has an error, records, max_wait_ms == 0, a diverging epoch, a snapshot id, or a high watermark newer than the follower's; otherwise it is parked; tests for each condition.

### [To Do] RAFT-0049 Bounded fetch purgatory completed by appends or high-watermark changes, expired by max wait
P0 · M3 · Task · deps: RAFT-0048 · ref: raft/src/main/java/org/apache/kafka/raft/internals/ThresholdPurgatory.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/FuturePurgatory.java @4.4.0
Done when: Parked fetches are keyed by fetch offset, completed when the leader appends past it or the high watermark moves, re-evaluated on completion, and expire after max_wait_ms returning the original empty response; resignation fails them with NOT_LEADER_OR_FOLLOWER; capacity is bounded by voters plus observers with a typed rejection; tests with the simulated clock.

### [To Do] RAFT-0050 Follower progress on the leader: end offset, last fetch and last caught-up timestamps
P0 · M3 · Task · deps: RAFT-0047 · ref: raft/src/main/java/org/apache/kafka/raft/LeaderState.java updateReplicaState, ReplicaState @4.4.0
Done when: Fetches update the remote replica state (ignored for negative replica ids; the leader's own id is an invariant violation); last caught-up time follows Kafka's rule against the leader end offset; voters affect the high watermark, observers do not; non-monotonic follower offsets are logged at warn; tests.

### [To Do] RAFT-0051 Follower Fetch request construction
P0 · M3 · Task · deps: RAFT-0017, RAFT-F15 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java buildFetchRequest @4.4.0
Done when: The follower sends cluster id, replica id and directory id, current leader epoch, fetch offset = log end offset, last fetched epoch, its high watermark, the driver-supplied fetch max wait and controller.quorum.fetch.max.bytes; golden encoding test.

### [To Do] RAFT-0052 Follower truncation on a diverging epoch, never below the high watermark
P0 · M3 · Task · deps: RAFT-0051, RAFT-0019 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchResponse @4.4.0; raft/src/main/java/org/apache/kafka/raft/RaftLog.java truncateToEndOffset @4.4.0
Done when: A response with a diverging epoch truncates the log to the end offset for that epoch (min with the local end offset for the epoch), updates the control-record state machine and refetches; a diverging offset below the follower's high watermark is a fatal invariant violation; tests with crafted divergent logs.

### [To Do] RAFT-0053 Follower append of fetched records and high-watermark update
P0 · M3 · Task · deps: RAFT-0051 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java appendAsFollower @4.4.0; raft/src/main/java/org/apache/kafka/raft/FollowerState.java updateHighWatermark @4.4.0
Done when: Records append as follower in the current epoch; corrupt or invalid batches are logged with the batch header hex and skipped until refetch; the follower's high watermark becomes the leader's (monotonic, empty when -1); voter followers flush after append; tests including a corrupt batch.

### [To Do] RAFT-0054 Fetch timeout: voter followers become Prospective; successful fetches reset the timer
P0 · M3 · Task · deps: RAFT-0053 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java pollFollowerAsVoter @4.4.0; raft/src/main/java/org/apache/kafka/raft/FollowerState.java hasFetchTimeoutExpired @4.4.0
Done when: A voter follower whose fetch timeout (controller.quorum.fetch.timeout.ms) expires becomes Prospective; every handled fetch or fetch-snapshot response resets it; observers never time out; tests with the simulated clock.

### [To Do] RAFT-0055 Fetch target selection: leader unless timed out or backing off, then any bootstrap server
P0 · M3 · Task · deps: RAFT-0054 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java maybeSendFetchToBestNode, maybeSendFetchToAnyBootstrap @4.4.0
Done when: When the leader's request times out or it is backing off, the follower fetches from a ready bootstrap server to discover a leader change; with a request in flight it waits; the poll deadline is the minimum of backoff and remaining fetch timeout; tests.

### [To Do] RAFT-0056 Leader discovery from Fetch responses in the same request
P0 · M3 · Task · deps: RAFT-0053, RAFT-0019 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java maybeHandleCommonResponse @4.4.0
Done when: A Fetch response naming a leader for the current epoch moves the node to Follower and, when the partition error is NONE, processes the records in the same response; tests for unattached voters and observers.

### [To Do] RAFT-0057 Leader appends, flushes and advances its end offset before releasing parked fetches
P0 · M3 · Task · deps: RAFT-0049, RAFT-F10 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java flushLeaderLog, updateLeaderEndOffsetAndTimestamp @4.4.0
Done when: The leader updates its end offset (so parked fetches can complete) before flushing, and the high watermark is advanced only from flushed offsets; a test with a slow simulated fsync shows no fetch reads beyond the flushed offset as committed.

### [To Do] RAFT-0058 Port Kafka's fetch, divergence and high-watermark client tests
P0 · M3 · Task · deps: RAFT-0052, RAFT-0055, RAFT-0056, RAFT-0057, RAFT-0050 · labels: test · ref: raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientFetchTest.java @4.4.0; raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientTest.java @4.4.0
Done when: Fetch-related cases from KafkaRaftClientTest at 4.4.0 (divergence, purgatory, HW propagation, fetch timeout) pass with attribution.

### [To Do] RAFT-0059 Benchmark leader fetch handling and follower append throughput
P2 · M3 · Task · deps: RAFT-0058 · labels: perf
Done when: Criterion benchmarks for tryCompleteFetchRequest and follower append on 1 MiB batches run in CI with regression thresholds; results recorded in the PR.

## [To Do] RAFT-F05 Raft log storage
P0 · M2 · Feature · deps: STOR-F08
Scope: The `__cluster_metadata-0` log on the storage crate.

### [To Do] RAFT-0060 Metadata log on the storage crate: __cluster_metadata-0 with its fixed topic id and MetadataLogConfig
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java createLog @4.4.0; raft/src/main/java/org/apache/kafka/raft/MetadataLogConfig.java @4.4.0
Done when: The raft log opens or creates __cluster_metadata-0 under metadata.log.dir with Kafka's fixed metadata topic id through the STOR-F08 API; MetadataLogConfig carries segment bytes/ms, max batch size and retention bytes/ms; time and size retention of segments are disabled (segments are deleted only behind snapshots); tests open, close and reopen.

### [To Do] RAFT-0061 append_as_leader: offsets assigned in the leader epoch, epoch cache updated
P0 · M2 · Task · deps: RAFT-0060, STOR-F11 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java appendAsLeader @4.4.0
Done when: Appends assign contiguous offsets, stamp the partition leader epoch and add an epoch cache entry on epoch change; an empty append is an invariant violation; returns first and last offset; unit tests.

### [To Do] RAFT-0062 append_as_follower: validate contiguity and epochs, typed corrupt-record error
P0 · M2 · Task · deps: RAFT-0060, STOR-F11 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java appendAsFollower @4.4.0
Done when: Follower appends keep the batches' offsets, reject non-contiguous offsets or epochs above the current epoch with a typed error carrying the batch header, and update the epoch cache from batches; tests with crafted batches.

### [To Do] RAFT-0063 Bounded reads with UNCOMMITTED and COMMITTED isolation and offset metadata
P0 · M2 · Task · deps: RAFT-0060 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java read @4.4.0; raft/src/main/java/org/apache/kafka/raft/Isolation.java @4.4.0
Done when: read(start, isolation, max_bytes) returns whole batches up to max bytes (at least one batch), bounded by the high watermark for COMMITTED, plus segment position metadata for the start offset; tests for both isolations and boundaries.

### [To Do] RAFT-0064 Last fetched epoch and end offset for epoch, including the snapshot fallback
P0 · M2 · Task · deps: RAFT-0061 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java lastFetchedEpoch, endOffsetForEpoch @4.4.0
Done when: last_fetched_epoch comes from the epoch cache, or from the latest snapshot when the log is empty at the snapshot offset (any other empty-log state is an invariant violation), else 0; end_offset_for_epoch returns the largest epoch <= requested and its end offset, using the earliest snapshot id at the log start; tests ported from KafkaRaftLogTest.

### [To Do] RAFT-0065 validate_offset_and_epoch: VALID, DIVERGING or SNAPSHOT
P0 · M2 · Task · deps: RAFT-0064 · ref: raft/src/main/java/org/apache/kafka/raft/RaftLog.java validateOffsetAndEpoch @4.4.0; raft/src/main/java/org/apache/kafka/raft/ValidOffsetAndEpoch.java @4.4.0
Done when: Offsets below the log start (or at it with a mismatched snapshot epoch) yield SNAPSHOT with the latest snapshot id; an epoch whose end offset is below the offset or a smaller found epoch yields DIVERGING with that end offset and epoch; otherwise VALID; a property test checks the result against a model log.

### [To Do] RAFT-0066 In-memory high watermark with offset metadata
P0 · M2 · Task · deps: RAFT-0063 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java updateHighWatermark, highWatermark @4.4.0
Done when: update_high_watermark clamps to [log start, log end], keeps segment position metadata, and is not checkpointed for the metadata log (recomputed from the quorum after restart); tests for clamping and restart.

### [To Do] RAFT-0067 Flush policy for the metadata log: flush before ack, optional active-segment force
P0 · M2 · Task · deps: RAFT-0061, RAFT-0062 · labels: nfr · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java flush @4.4.0
Done when: flush(force_active_segment) fsyncs through the storage fs trait; voters flush before fetching and the leader before advancing its end offset; a crash test with the fault-injecting fs loses no flushed record.

### [To Do] RAFT-0068 Truncation: truncate_to refuses offsets below the high watermark; truncate to diverging end offset
P0 · M3 · Task · deps: RAFT-0065, RAFT-0066, STOR-F12 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java truncateTo @4.4.0; raft/src/main/java/org/apache/kafka/raft/RaftLog.java truncateToEndOffset @4.4.0
Done when: truncate_to(offset) below the high watermark is an invariant violation; truncate_to_end_offset(diverging) truncates to min(diverging end offset, local end offset for that epoch) and updates the epoch cache; tests.

### [To Do] RAFT-0069 Crash-consistency tests for the metadata log
P0 · M2 · Task · deps: RAFT-0067, STOR-F20 · labels: test
Done when: With the fault-injecting fs, crashes at every write point during leader and follower appends recover to a log whose committed prefix is intact and whose epoch cache matches the batches; runs in CI.

## [To Do] RAFT-F06 Snapshots
P0 · M2 · Feature · deps: RAFT-F05, REC-F11
Scope: Snapshot writer and reader with header/footer, retention of old snapshots; FetchSnapshot in M3.

### [To Do] RAFT-0070 Snapshot file naming: <20-digit offset>-<10-digit epoch>.checkpoint, .part and .deleted
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/snapshot/Snapshots.java @4.4.0; raft/src/main/java/org/apache/kafka/snapshot/SnapshotPath.java @4.4.0
Done when: Snapshot ids format and parse exactly as Kafka's names; partial (.checkpoint.part) and deleted (.checkpoint.deleted) suffixes are recognized; unknown names are ignored; round-trip and invalid-name tests.

### [To Do] RAFT-0071 Raw snapshot writer: append to a .part file, freeze with fsync and atomic rename
P0 · M2 · Task · deps: RAFT-0070, STOR-F01 · ref: raft/src/main/java/org/apache/kafka/snapshot/FileRawSnapshotWriter.java @4.4.0
Done when: Writes go to a unique .part file; freeze fsyncs, renames to the final name and fsyncs the directory; a frozen writer rejects appends; close without freeze deletes the partial file; crash tests show only complete snapshots survive.

### [To Do] RAFT-0072 Raw snapshot reader with size and positional slices
P0 · M2 · Task · deps: RAFT-0070 · ref: raft/src/main/java/org/apache/kafka/snapshot/FileRawSnapshotReader.java @4.4.0
Done when: A reader opens a frozen snapshot, reports its size and returns byte slices at a position up to a max size (used by FetchSnapshot); positions beyond the size are rejected with a typed error; tests.

### [To Do] RAFT-0073 Records snapshot writer: header record, batched records, kraft.version and voters, footer
P0 · M2 · Task · deps: RAFT-0071, REC-F04 · ref: raft/src/main/java/org/apache/kafka/snapshot/RecordsSnapshotWriter.java @4.4.0; clients/src/main/resources/common/message/SnapshotHeaderRecord.json @4.4.0; clients/src/main/resources/common/message/SnapshotFooterRecord.json @4.4.0
Done when: The writer emits SnapshotHeaderRecord (version, last contained log timestamp) first, the KRaftVersionRecord and VotersRecord when kraft.version >= 1, then user records in batches no larger than the max batch size, and SnapshotFooterRecord on freeze; tests decode the output and compare with Kafka's layout.

### [To Do] RAFT-0074 Records snapshot reader validating header and footer as untrusted input
P0 · M2 · Task · deps: RAFT-0072 · labels: decoder · ref: raft/src/main/java/org/apache/kafka/snapshot/RecordsSnapshotReader.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/RecordsIterator.java @4.4.0
Done when: The reader iterates batches, yields control records (header, kraft.version, voters, footer) separately from user records, errors on a missing header/footer or a truncated batch without panicking, and bounds batch sizes before allocating; tests with corrupted files.

### [To Do] RAFT-0075 Fuzz target for snapshot decoding
P0 · M2 · Task · deps: RAFT-0074, TEST-F07 · labels: decoder, security, test
Done when: fuzz/fuzz_targets/raft_snapshot_reader feeds arbitrary bytes to the records snapshot reader; no panics, hangs or unbounded allocation in the CI short run; seed corpus from generated snapshots.

### [To Do] RAFT-0076 create_snapshot validation: id at or below the high watermark and consistent with the log
P0 · M2 · Task · deps: RAFT-0073, RAFT-F05 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java createNewSnapshot, createNewSnapshotUnchecked @4.4.0
Done when: Creating a snapshot for an id above the high watermark or whose epoch is not valid for that offset (validate_offset_and_epoch) is rejected with a typed error; an id below the log start offset returns None; the unchecked variant (for fetched snapshots) skips validation; tests.

### [To Do] RAFT-0077 Snapshot recovery at startup: delete partial and deleted files, index frozen snapshots
P0 · M2 · Task · deps: RAFT-0071, RAFT-0072 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java recoverSnapshots @4.4.0
Done when: On open, .part and .deleted files are removed, frozen snapshots are indexed by id, and a log start offset without a matching snapshot (other than 0) is an inconsistent-state error; crash tests cover each leftover file kind.

### [To Do] RAFT-0078 Advance the log start offset behind a snapshot and delete older segments and snapshots
P0 · M2 · Task · deps: RAFT-0077 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java deleteBeforeSnapshot, onSnapshotFrozen @4.4.0
Done when: delete_before_snapshot(id) only proceeds when id <= high watermark and a snapshot exists; it advances the log start offset to the snapshot end offset, deletes older segments, and renames older snapshots to .deleted before removal; tests.

### [To Do] RAFT-0079 Snapshot retention by metadata.max.retention.bytes and metadata.max.retention.ms
P0 · M2 · Task · deps: RAFT-0078 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java maybeClean, cleanSnapshotsRetentionMs @4.4.0
Done when: maybe_clean deletes the oldest snapshot (and log before the next one) while total size exceeds retention bytes or its timestamp is older than retention ms, never deleting the latest snapshot; each deletion logs a reason (retention_size_breach, retention_ms_breach); tests with both limits and -1 disabling.

### [To Do] RAFT-0080 Full truncation to the latest snapshot when it is ahead of the log
P0 · M2 · Task · deps: RAFT-0078 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftLog.java truncateToLatestSnapshot @4.4.0
Done when: truncate_to_latest_snapshot truncates fully and restarts the log at the snapshot offset when the latest snapshot's epoch or offset is ahead of the log, forgets older snapshots and reports whether it truncated; tests.

### [To Do] RAFT-0081 FetchSnapshot leader handling with Kafka's errors
P0 · M3 · Task · deps: RAFT-0072, RAFT-0018, RAFT-0038 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchSnapshotRequest @4.4.0
Done when: The leader validates cluster id, partition (UNKNOWN_TOPIC_OR_PARTITION), leader-only rules, rejects missing or bootstrap (0-0) snapshots with SNAPSHOT_NOT_FOUND and positions outside [0, size) with POSITION_OUT_OF_RANGE, returns a slice up to min(max bytes, size) with size and position, and counts the fetch toward check-quorum; tests per error.

### [To Do] RAFT-0082 FetchSnapshot follower download, resume by position and install
P0 · M3 · Task · deps: RAFT-0081, RAFT-0080, RAFT-0051 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleFetchSnapshotResponse, buildFetchSnapshotRequest @4.4.0
Done when: A Fetch response with a snapshot id starts an unchecked snapshot writer; FetchSnapshot requests ask from the current size up to controller.quorum.fetch.snapshot.max.bytes; SNAPSHOT_NOT_FOUND or a missing id resets to normal fetching; mismatched id or position is an invariant violation; when complete the snapshot is frozen, the log fully truncated to it and the high watermark updated; tests including a leader deleting the snapshot mid-download.

### [To Do] RAFT-0083 Port Kafka's snapshot client tests
P0 · M3 · Task · deps: RAFT-0082, RAFT-0079, RAFT-F10 · labels: test · ref: raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientSnapshotTest.java @4.4.0
Done when: Cases from KafkaRaftClientSnapshotTest at 4.4.0 (snapshot creation, listener loading, FetchSnapshot download and errors) pass with attribution.

### [To Do] RAFT-0084 docs/formats entry for metadata snapshots and the metadata log directory layout
P1 · M2 · Task · deps: RAFT-0073 · labels: docs
Done when: docs/formats/raft-snapshot.md documents file naming, header/footer records, control records and the batch layout with a hex example matching a generated file.

## [To Do] RAFT-F07 Dynamic quorum
P0 · M3 · Feature · deps: RAFT-F02, RAFT-F04
Scope: KIP-853: AddRaftVoter, RemoveRaftVoter, UpdateRaftVoter, kraft.version, voter set records.

### [To Do] RAFT-0085 Shared voter-change preconditions: committed epoch, kraft.version 1, one change at a time
P0 · M3 · Task · deps: RAFT-0003, RAFT-F15 · ref: raft/src/main/java/org/apache/kafka/raft/internals/AddVoterHandler.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/ChangeVoterHandlerState.java @4.4.0
Done when: Add, remove and update voter return REQUEST_TIMED_OUT until the leader's high watermark is known in its epoch, UNSUPPORTED_VERSION when kraft.version < 1 (update stores in memory instead), and REQUEST_TIMED_OUT while an uncommitted voter change exists; tests per precondition.

### [To Do] RAFT-0086 AddRaftVoter: validation, ApiVersions probe of the new voter, catch-up check and VotersRecord
P0 · M3 · Task · deps: RAFT-0085 · ref: raft/src/main/java/org/apache/kafka/raft/internals/AddVoterHandler.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleAddVoterRequest @4.4.0; raft/src/main/java/org/apache/kafka/raft/LeaderState.java isReplicaCaughtUp @4.4.0
Done when: DUPLICATE_VOTER for an existing id, INVALID_REQUEST without a valid voter or endpoint; the leader probes the new voter's default listener with ApiVersions, rejects unsupported kraft.version with INVALID_REQUEST, requires the voter to have fetched and caught up within the last hour (else REQUEST_TIMED_OUT), appends the new VotersRecord and responds once it commits with the new majority; tests per step.

### [To Do] RAFT-0087 RemoveRaftVoter: append the reduced voter set and resign if the leader was removed
P0 · M3 · Task · deps: RAFT-0085 · ref: raft/src/main/java/org/apache/kafka/raft/internals/RemoveVoterHandler.java @4.4.0
Done when: Removing a voter appends the VotersRecord without it, responds after commit (REQUEST_TIMED_OUT on timeout), and the leader resigns when it is no longer a voter; removing a replica key that is not a voter returns VOTER_NOT_FOUND; tests.

### [To Do] RAFT-0088 UpdateRaftVoter: listener and kraft.version range updates for an existing voter
P0 · M3 · Task · deps: RAFT-0085 · ref: raft/src/main/java/org/apache/kafka/raft/internals/UpdateVoterHandler.java @4.4.0
Done when: The leader validates the voter exists (VOTER_NOT_FOUND), still supports the finalized kraft.version and the default listener (INVALID_REQUEST), then appends an updated VotersRecord (kraft.version >= 1) or updates in-memory voters (kraft.version 0); tests for both kraft versions.

### [To Do] RAFT-0089 Pending voter change expiry and completion on high-watermark advance
P0 · M3 · Task · deps: RAFT-0086, RAFT-0087 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java maybeNotifyVoterHandlerOnHWmUpdate @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/ChangeVoterHandlerState.java @4.4.0
Done when: A pending add/remove completes when the high watermark passes the VotersRecord offset and fails with REQUEST_TIMED_OUT after the request timeout; the leader's poll deadline includes the expiry; tests.

### [To Do] RAFT-0090 Voters send UpdateRaftVoter when their listeners differ from the voter set
P0 · M3 · Task · deps: RAFT-0088, RAFT-0051 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java shouldSendUpdateVoteRequest, pollFollowerAsVoter @4.4.0
Done when: A voter follower sends UpdateRaftVoter to the leader when kraft.version >= 1 and its local listeners or version range differ from the voter set, or once per new leader when kraft.version is 0; the periodic timer resets after sending; tests.

### [To Do] RAFT-0091 Auto-join (controller.quorum.auto.join.enable): observers add themselves or remove stale entries
P0 · M3 · Task · deps: RAFT-0086, RAFT-0087, RAFT-F08 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java pollFollowerAsObserver, shouldSendAddOrRemoveVoterRequest @4.4.0
Done when: An observer controller with auto-join enabled and kraft.version >= 1 sends RemoveRaftVoter for a voter entry with its id but another directory id, otherwise AddRaftVoter for itself, rate limited by the update-voter period; disabled by default; tests.

### [To Do] RAFT-0092 kraft.version upgrade from 0 to 1 with validation and control records
P0 · M3 · Task · deps: RAFT-0085 · ref: raft/src/main/java/org/apache/kafka/raft/LeaderState.java maybeAppendUpgradedKRaftVersion @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/KRaftVersionUpgrade.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java upgradeKRaftVersion @4.4.0
Done when: upgrade_kraft_version(epoch, version, validate_only) rejects stale epochs, concurrent upgrades to another version, downgrades and voter sets where any voter lacks support or a directory id (InvalidUpdateVersion); on success it appends KRaftVersionRecord plus the in-memory voter set as VotersRecord; validate_only appends nothing; tests per rule.

### [To Do] RAFT-0093 High watermark across voter set changes uses the latest (uncommitted) voter set
P0 · M3 · Task · deps: RAFT-0086, RAFT-0087 · ref: raft/src/main/java/org/apache/kafka/raft/LeaderState.java updateVoterAndObserverStates @4.4.0
Done when: The leader recomputes voter/observer states from the last voter set on each local append, so a removed voter stops counting and an added voter counts once its VotersRecord is in the log; a temporary HW regression is only logged; tests for add then remove.

### [To Do] RAFT-0094 Uncommitted voter change and voter count signals for metrics
P1 · M3 · Task · deps: RAFT-0089 · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateUncommittedVoterChange @4.4.0
Done when: The state machine exposes whether an uncommitted voter change exists and the current number of voters/observers so RAFT-F14 can publish uncommitted-voter-change, number-of-voters and number-of-observers; tests.

### [To Do] RAFT-0095 Port Kafka's reconfiguration tests
P0 · M3 · Task · deps: RAFT-0091, RAFT-0092, RAFT-0093, RAFT-0090 · labels: test · ref: raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientReconfigTest.java @4.4.0; raft/src/test/java/org/apache/kafka/raft/KafkaRaftClientAutoJoinTest.java @4.4.0
Done when: Cases from KafkaRaftClientReconfigTest at 4.4.0 (add, remove, update, auto-join, kraft.version upgrade) pass with attribution.

## [To Do] RAFT-F08 Observers
P0 · M3 · Feature · deps: RAFT-F04
Scope: Brokers replicating the metadata log as non-voting observers.

### [To Do] RAFT-0096 Observer role: non-voters never time out into elections and never vote
P0 · M3 · Task · deps: RAFT-0009 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumState.java isObserver, transitionToUnattached @4.4.0
Done when: A node whose (id, directory id) is not in the voter set is an observer: its election timeout is infinite, it cannot enter Prospective/Candidate/Leader, and Vote requests to it are handled by the normal rules but it never campaigns; tests.

### [To Do] RAFT-0097 Observers discover the leader through controller.quorum.bootstrap.servers
P0 · M3 · Task · deps: RAFT-0096, RAFT-0055 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java pollUnattachedCommon, maybeSendFetchToAnyBootstrap @4.4.0
Done when: An Unattached observer fetches from a ready bootstrap server, learns the leader from the response and becomes Follower; when the leader request times out it falls back to bootstrap servers; tests with rotating bootstrap nodes.

### [To Do] RAFT-0098 Observers skip fsync unless they can become voters
P0 · M3 · Task · deps: RAFT-0096, RAFT-0053 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java appendAsFollower @4.4.0
Done when: Observer followers append without flushing unless can_become_voter is set (controllers that may be added); voters always flush; tests assert flush actions per role.

### [To Do] RAFT-0099 Leader-side observer tracking with session timeout cleanup
P0 · M3 · Task · deps: RAFT-0050 · ref: raft/src/main/java/org/apache/kafka/raft/LeaderState.java clearInactiveObservers, getOrCreateReplicaState @4.4.0
Done when: The leader keeps observer replica states keyed by replica key, does not count them for the high watermark, and removes observers inactive for 300000 ms (OBSERVER_SESSION_TIMEOUT_MS) when listing them; tests with the simulated clock.

### [To Do] RAFT-0100 Brokers run the raft client as observers of the metadata quorum
P0 · M3 · Task · deps: RAFT-0097, RAFT-0098, TEST-F06 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java initialize @4.4.0
Done when: A broker-only node starts the raft client with no voter role, follows the leader and delivers committed batches to the metadata loader; a 3-controller/2-broker simulation shows brokers catching up and following leader changes.

## [To Do] RAFT-F09 Quorum state persistence
P0 · M2 · Feature · deps: STOR-F10
Scope: `quorum-state` file with atomic writes.

### [To Do] RAFT-0101 quorum-state JSON format v0 and v1 with data_version validation
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/FileQuorumStateStore.java @4.4.0; raft/src/main/resources/common/message/QuorumStateData.json @4.4.0
Done when: The file encodes leaderId, leaderEpoch, votedId, votedDirectoryId (v1) and currentVoters (v0 only) plus data_version; reading rejects a missing data_version or one outside [0, 1] with a typed error; golden files written by Kafka 4.4.0 decode identically; docs/formats/quorum-state.md documents it.

### [To Do] RAFT-0102 Atomic quorum-state writes: tmp file, fsync, rename, directory fsync
P0 · M2 · Task · deps: RAFT-0101, STOR-F10 · ref: raft/src/main/java/org/apache/kafka/raft/FileQuorumStateStore.java writeElectionStateToFile @4.4.0
Done when: Writes go to quorum-state.tmp, are fsynced, renamed over quorum-state and the directory fsynced through the fs trait; a leftover .tmp is deleted at startup; crash tests at each step leave either the old or the new state.

### [To Do] RAFT-0103 QuorumStateStore trait and file implementation driven by persist actions
P0 · M2 · Task · deps: RAFT-0102 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumStateStore.java @4.4.0
Done when: The driver executes PersistElectionState actions through a QuorumStateStore (file implementation plus in-memory test double); an absent file reads as epoch 0 with no leader or vote; write version is v1 when kraft.version >= 1 else v0; tests.

### [To Do] RAFT-0104 Fuzz target for quorum-state parsing
P0 · M2 · Task · deps: RAFT-0101, TEST-F07 · labels: decoder, security, test
Done when: fuzz/fuzz_targets/raft_quorum_state feeds arbitrary bytes to the parser; no panics or unbounded allocation in the CI short run.

## [To Do] RAFT-F10 Raft client interface
P0 · M2 · Feature · deps: RAFT-F01
Scope: Append scheduling, commit and snapshot-load listeners, leader-change notifications.

### [To Do] RAFT-0105 BatchAccumulator: append in the leader epoch, linger, size-bounded batches and drain
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/internals/BatchAccumulator.java append, drain @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/BatchBuilder.java @4.4.0
Done when: append(epoch, records) rejects older epochs (NotLeader) and newer ones (invariant), bounds completed batches (typed buffer-full error), rolls batches at the max batch size, drains after controller.quorum.append.linger.ms or when a batch fills, and returns the last offset; tests with the simulated clock.

### [To Do] RAFT-0106 Bounded batch memory pool for the accumulator
P0 · M2 · Task · deps: RAFT-0105 · labels: nfr · ref: raft/src/main/java/org/apache/kafka/raft/internals/BatchMemoryPool.java @4.4.0
Done when: Batch buffers come from a pool bounded by max batches times max batch size; allocation failure surfaces as BufferAllocation error to the caller; released batches return buffers; tests for exhaustion.

### [To Do] RAFT-0107 prepare_append and schedule_prepared_append: delayed drain for atomic multi-batch writes
P0 · M2 · Task · deps: RAFT-0105 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java prepareAppend, schedulePreparedAppend @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/BatchAccumulator.java allowDrain @4.4.0
Done when: prepare_append records a drain fence at the first prepared offset so records are not drained until schedule_prepared_append; appends with no records or when not leader fail with typed errors; tests that prepared records are invisible to the log until scheduled.

### [To Do] RAFT-0108 Control batches in the accumulator: leader change, voters, kraft.version
P0 · M2 · Task · deps: RAFT-0105, REC-F11 · ref: raft/src/main/java/org/apache/kafka/raft/internals/BatchAccumulator.java appendControlMessages @4.4.0; raft/src/main/java/org/apache/kafka/raft/LeaderState.java appendStartOfEpochControlRecords @4.4.0
Done when: appendControlMessages builds control batches (LeaderChangeMessage, VotersRecord, KRaftVersionRecord) at the next offset, forcing a drain; the leader writes a start-of-epoch batch with LeaderChangeMessage (leader, voters, granting voters) and, for kraft.version >= 1, the voter set when it was never written or the leader's endpoints changed; tests decode the batches.

### [To Do] RAFT-0109 Leader drain loop: append drained batches, flush, advance end offset and HW
P0 · M2 · Task · deps: RAFT-0107, RAFT-0108, RAFT-F05 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java maybeAppendBatches, appendBatch @4.4.0
Done when: On poll the leader appends drained batches as leader, flushes, updates its local end offset and the high watermark, and reports the next drain deadline; a single-voter test commits a batch within one linger period.

### [To Do] RAFT-0110 Listener registration and per-listener progress tracking
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java register, processRegistration @4.4.0
Done when: register/unregister take effect on the next poll; each listener tracks its next offset and last acknowledged leader; unregistering mid-delivery stops further callbacks; tests.

### [To Do] RAFT-0111 handle_commit: deliver committed batches up to the high watermark in order
P0 · M2 · Task · deps: RAFT-0110, RAFT-0109 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java updateListenersProgress, maybeFireHandleCommit @4.4.0
Done when: Listeners receive batches (base offset, epoch, append time, size, records) strictly below the high watermark in offset order; the leader delivers its own appends from memory when the listener is caught up, otherwise reads the log; control records are not delivered as user records; tests.

### [To Do] RAFT-0112 handle_load_snapshot when a listener's next offset is below the log start
P0 · M2 · Task · deps: RAFT-0111, RAFT-F06 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java updateListenersProgress, latestSnapshot @4.4.0
Done when: A listener behind the log start (or new with a snapshot present) first receives the latest snapshot reader, then commits after it; tests with a snapshot at offset N and listeners at 0 and N.

### [To Do] RAFT-0113 handle_leader_change notifications once per epoch per listener
P0 · M2 · Task · deps: RAFT-0110 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java maybeFireLeaderChange @4.4.0
Done when: Listeners are told about each new LeaderAndEpoch once; a new leader is announced to a listener only after it has caught up to the epoch start offset; tests across leader changes.

### [To Do] RAFT-0114 Raft client API surface: leader_and_epoch, high_watermark, latest_snapshot_id, kraft_version, voter_node
P0 · M2 · Task · deps: RAFT-0110 · labels: docs · ref: raft/src/main/java/org/apache/kafka/raft/RaftClient.java @4.4.0
Done when: The public API mirrors RaftClient's read accessors with documented consistency (values from the last poll); unit tests; the API is what META-F04 and CTRL-F01 consume.

### [To Do] RAFT-0115 create_snapshot API for listeners at a committed offset
P0 · M2 · Task · deps: RAFT-0114, RAFT-F06 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java createSnapshot @4.4.0
Done when: Listeners can create a snapshot writer for an id at or below the high watermark (RAFT-F06 validation) with the last contained log time; frozen snapshots trigger log cleaning; tests.

### [To Do] RAFT-0116 Listener contract tests: ordering, no gaps, snapshot-then-commit, leader change ordering
P0 · M2 · Task · deps: RAFT-0112, RAFT-0113 · labels: test
Done when: A property test with random appends, leader changes and snapshots asserts each listener sees a gapless, ordered stream (snapshot then batches after it) and leader changes in epoch order.

## [To Do] RAFT-F11 Raft driver
P0 · M2 · Feature · deps: RAFT-F10, SRV-F02, NET-F10
Scope: Tokio driver: timers, network, log I/O, flight recorder.

### [To Do] RAFT-0117 Raft driver task: owns the client, polls until the next deadline and executes actions
P0 · M2 · Task · deps: RAFT-F09 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClientDriver.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java poll @4.4.0
Done when: A supervised, named Tokio task feeds inbound messages, ticks and API calls into the sans-I/O client, executes returned actions in order (persist before send), and sleeps until the returned deadline or the next input; tests with the simulated clock.

### [To Do] RAFT-0118 Bounded inbound message queue with backpressure
P0 · M2 · Task · deps: RAFT-0117 · labels: nfr · ref: raft/src/main/java/org/apache/kafka/raft/internals/BlockingMessageQueue.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/RaftMessageQueue.java @4.4.0
Done when: Inbound requests and responses go through a bounded channel sized from config; when full, the network layer stops reading from controller connections (muting); depth, capacity and rejections are exported; tests for saturation.

### [To Do] RAFT-0119 Network channel over the inter-node client with correlation ids and timeouts
P0 · M2 · Task · deps: RAFT-0117 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaNetworkChannel.java @4.4.0
Done when: Outbound raft requests go through NET-F10 to the controller listener of the target node with per-request timeout; responses or transport errors come back as inputs carrying the correlation id; unknown destinations resolve from the voter set or bootstrap servers; tests with a fake transport.

### [To Do] RAFT-0120 Log and snapshot I/O executed off the async workers
P0 · M2 · Task · deps: RAFT-0117, STOR-F18 · labels: nfr
Done when: Append, flush, read and snapshot actions run on the storage I/O execution model (STOR-F18) and report completion as inputs; no blocking call runs on a Tokio worker (checked by the clippy disallowed-methods config); tests.

### [To Do] RAFT-0121 Inbound raft requests from the request router are answered via deferred responses
P0 · M2 · Task · deps: RAFT-0118 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handle @4.4.0
Done when: Vote, BeginQuorumEpoch, EndQuorumEpoch, Fetch, FetchSnapshot, DescribeQuorum and voter-change requests handed over by API handlers return a future that the driver completes when the state machine emits the response (including parked fetches); dropped connections cancel cleanly; tests.

### [To Do] RAFT-0122 Raft flight recorder: state transitions, elections, truncations, snapshot installs
P0 · M2 · Task · deps: RAFT-0117, RAFT-0014, OBS-F06 · labels: observability
Done when: The driver records every StateTransition and key decision (vote granted/rejected with reason, truncation offsets, snapshot install, HW first set per epoch) in the quorum flight recorder (256 entries), dumps it on error and serves it at /debug/raft; tests read it back.

### [To Do] RAFT-0123 Driver shutdown: graceful resignation with timeout, then close log and quorum store
P0 · M2 · Task · deps: RAFT-0117, SRV-F02 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java shutdown @4.4.0; raft/src/main/java/org/apache/kafka/raft/KafkaRaftClientDriver.java initiateShutdown @4.4.0
Done when: Shutdown calls the client's graceful shutdown with a bounded timeout, waits for completion or expiry, then flushes and closes the log; it runs last in the node shutdown order and logs start, end and duration; tests.

### [To Do] RAFT-0124 Fatal error handling: invariant violations abort with the flight recorder dumped
P0 · M2 · Task · deps: RAFT-0122, SRV-F16
Done when: Errors classified as invariant violations (truncation below HW, inconsistent leader) panic with `expect("invariant: ...")` and the panic hook dumps the raft flight recorder; recoverable I/O errors on the metadata log are fatal for the node with a clear error log; tests check classification.

### [To Do] RAFT-0125 Optional input journaling for the raft state machine
P2 · M3 · Task · deps: RAFT-0117, SRV-F15 · labels: observability
Done when: With journaling enabled, every input to the raft client is recorded (metadata only, no record payloads) and the replay tool reproduces the same actions; a test replays a recorded election.

## [To Do] RAFT-F12 DescribeQuorum
P1 · M3 · Feature · deps: RAFT-F04
Scope: Quorum state reporting for `kafka-metadata-quorum.sh`.

### [To Do] RAFT-0126 DescribeQuorum on the leader: leader id, epoch, high watermark, voter and observer states
P1 · M3 · Task · deps: RAFT-0050, RAFT-0099 · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java handleDescribeQuorumRequest @4.4.0; raft/src/main/java/org/apache/kafka/raft/RaftUtil.java singletonDescribeQuorumResponse @4.4.0
Done when: Only the leader answers (others NOT_LEADER_OR_FOLLOWER); a wrong partition yields UNKNOWN_TOPIC_OR_PARTITION; the response lists voters and active observers with log end offsets; tests.

### [To Do] RAFT-0127 DescribeQuorum v1 timestamps and v2 directory ids and node endpoints
P1 · M3 · Task · deps: RAFT-0126 · ref: clients/src/main/resources/common/message/DescribeQuorumResponse.json @4.4.0
Done when: v1 adds last fetch and last caught-up timestamps (the leader reports the current time for itself), v2 adds replica directory ids and the nodes list with listeners; golden tests per version.

### [To Do] RAFT-0128 kafka-metadata-quorum.sh describe --status and --replication compatibility
P1 · M3 · Task · deps: RAFT-0127, TEST-F08 · labels: test
Done when: The compatibility harness runs kafka-metadata-quorum.sh against a 3-controller cluster and the output fields match Kafka's for status and replication views.

## [To Do] RAFT-F13 Raft simulation testing
P0 · M3 · Feature · deps: RAFT-F02, RAFT-F04, TEST-F06
Scope: Election safety, log matching, leader completeness and state machine safety under faults.

### [To Do] RAFT-0129 Raft simulation scaffold: N voters plus observers on the simulated network and disk
P0 · M3 · Task · deps: none · labels: test
Done when: A sim test builds 1, 3 and 5 voter quorums with observers using TEST-F06, drives clients with appends, prints the seed and replay command, and runs a fixed seed set in CI plus random seeds.

### [To Do] RAFT-0130 Invariant: election safety (at most one leader per epoch)
P0 · M3 · Task · deps: RAFT-0129 · labels: test
Done when: A checker after every step asserts no two nodes are leader in the same epoch across all seeds and fault plans (drops, delays, partitions, crashes).

### [To Do] RAFT-0131 Invariant: log matching (same offset and epoch implies identical prefix)
P0 · M3 · Task · deps: RAFT-0129 · labels: test
Done when: A checker compares logs of all replicas: if two logs have a batch with the same offset and epoch, all preceding batches are identical.

### [To Do] RAFT-0132 Invariant: leader completeness and state machine safety for committed records
P0 · M3 · Task · deps: RAFT-0129 · labels: test
Done when: Every record committed (below some node's HW) appears at the same offset in every later leader's log and is delivered identically to every listener; no listener ever sees a committed record change.

### [To Do] RAFT-0133 Invariant: high watermark monotonic per node and never ahead of a majority
P0 · M3 · Task · deps: RAFT-0129 · labels: test
Done when: Each node's observed HW never decreases (outside voter set changes) and the leader's HW is replicated on a majority of voters at all times.

### [To Do] RAFT-0134 Invariant: at most one persisted vote per node per epoch across crashes
P0 · M3 · Task · deps: RAFT-0129 · labels: test
Done when: Nodes crash and restart between vote grant and response; the checker asserts the persisted quorum-state never shows two different voted keys for one epoch.

### [To Do] RAFT-0135 Scenario: a partitioned voter rejoining does not disrupt a stable leader (pre-vote)
P0 · M3 · Task · deps: RAFT-0130 · labels: test
Done when: A voter isolated for many election timeouts rejoins; the leader's epoch does not change; with pre-vote disabled via a v0-only peer the expected disruption is observed.

### [To Do] RAFT-0136 Scenario: an isolated leader resigns via check quorum and the majority elects a new one
P0 · M3 · Task · deps: RAFT-0132 · labels: test
Done when: The old leader becomes Resigned within the check-quorum timeout and later follows the new leader; no committed data is lost.

### [To Do] RAFT-0137 Liveness: a leader is elected within a bounded number of election timeouts after healing
P0 · M3 · Task · deps: RAFT-0130 · labels: test
Done when: After faults stop, a leader with all voters following is elected within 10 election timeouts for every seed; failures print the flight recorders.

### [To Do] RAFT-0138 Scenario: lagging followers and observers catch up through FetchSnapshot under faults
P0 · M3 · Task · deps: RAFT-0132 · labels: test
Done when: With aggressive snapshot retention, a follower down long enough to fall behind the log start installs a snapshot and converges; invariants hold.

### [To Do] RAFT-0139 Scenario: add and remove voters under faults keep safety
P0 · M3 · Task · deps: RAFT-0132, RAFT-F07 · labels: test
Done when: Voters are added and removed one at a time while messages drop and nodes crash; election safety and completeness hold and every change either commits or times out.

### [To Do] RAFT-0140 Scenario: fsync loss and torn writes on minority voters
P0 · M3 · Task · deps: RAFT-0132, TEST-F03 · labels: test
Done when: Disk faults on a minority of voters never lose a committed record; a node that lost unflushed data rejoins correctly.

### [To Do] RAFT-0141 Nightly long-run raft simulation with thousands of seeds
P1 · M3 · Task · deps: RAFT-0137, RAFT-0138, RAFT-0139, RAFT-0140, RAFT-0135, RAFT-0136, RAFT-0133, RAFT-0134, RAFT-0131 · labels: test
Done when: A nightly job runs every raft sim test with thousands of seeds; failing seeds are reported with replay commands and added to the fixed set.

## [To Do] RAFT-F14 Raft metrics and events
P1 · M2 · Feature · deps: RAFT-F11, OBS-F04
Scope: Kafka's raft metrics, leader changes, commit latency, events with reasons.

### [To Do] RAFT-0142 Raft event catalog with reasons in events.rs
P1 · M2 · Task · deps: OBS-F07 · labels: observability
Done when: alluvium-raft's events.rs lists raft_state_changed, raft_leader_changed, raft_vote_granted, raft_vote_rejected, raft_log_truncated, raft_snapshot_installed, raft_snapshot_deleted and raft_high_watermark_initialized with fields and reason enums; the driver logs them at info (vote decisions at debug); docs/events.md generation includes them.

### [To Do] RAFT-0143 Metrics current-state, current-leader and current-epoch
P1 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java @4.4.0
Done when: Gauges alluvium_raft_current_state (one-hot by state name), alluvium_raft_current_leader and alluvium_raft_current_epoch update on every transition; tests assert values after an election.

### [To Do] RAFT-0144 Metrics current-vote and current-vote-directory-id
P1 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java @4.4.0
Done when: Gauge alluvium_raft_current_vote holds the voted id (-1 when none) and an info metric with a single series carries the voted directory id (zero UUID when none, as in Kafka); both update on vote transitions; tests.

### [To Do] RAFT-0145 Metrics high-watermark, log-end-offset and log-end-epoch
P1 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateLogEnd @4.4.0
Done when: Gauges update on HW changes and appends/truncations; tests compare with the client state.

### [To Do] RAFT-0146 Metric commit-latency avg and max
P1 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateCommitLatency @4.4.0
Done when: A histogram alluvium_raft_commit_latency_seconds records append-to-commit latency on the leader (Kafka commit-latency-avg/max); tests with the simulated clock.

### [To Do] RAFT-0147 Metric election-latency avg and max
P1 · M3 · Task · deps: RAFT-0033 · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java maybeUpdateElectionLatency @4.4.0
Done when: A histogram alluvium_raft_election_latency_seconds records election start to leader; tests.

### [To Do] RAFT-0148 Metrics append-records-rate and fetch-records-rate
P1 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateAppendRecords, updateFetchedRecords @4.4.0
Done when: Counters alluvium_raft_append_records_total and alluvium_raft_fetch_records_total count records appended as leader and fetched as follower; tests.

### [To Do] RAFT-0149 Metric poll-idle-ratio-avg
P1 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updatePollStart @4.4.0
Done when: The driver measures time idle waiting for input versus processing and exports the average idle ratio; tests with the simulated clock.

### [To Do] RAFT-0150 Metrics number-of-voters, number-of-observers and uncommitted-voter-change
P1 · M3 · Task · deps: RAFT-0094 · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateNumVoters, updateNumObservers @4.4.0
Done when: Gauges reflect the leader's voter/observer counts and pending voter change flag; observers metric only on the leader; tests.

### [To Do] RAFT-0151 Metric number-unknown-voter-connections
P2 · M2 · Task · deps: none · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java updateNumUnknownVoterConnections @4.4.0
Done when: The gauge reports voters whose endpoints are unknown for the controller listener (0 with static voters); tests.

### [To Do] RAFT-0152 Leader-only metric registration and removal on leadership change
P1 · M2 · Task · deps: RAFT-0146 · labels: observability · ref: raft/src/main/java/org/apache/kafka/raft/internals/KafkaRaftMetrics.java addLeaderMetrics, removeLeaderMetrics @4.4.0
Done when: Leader-only metrics are registered on becoming leader and removed on losing leadership so scrapes never show stale values; tests across a leader change.

## [To Do] RAFT-F15 Quorum bootstrap
P0 · M2 · Feature · deps: SRV-F12
Scope: Standalone and initial-controllers bootstrap, cluster id checks.

### [To Do] RAFT-0153 Static voters: parse controller.quorum.voters (id@host:port) into a voter set
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java parseVoterConnections @4.4.0
Done when: Entries parse into voter ids and endpoints on the first controller listener; malformed entries, duplicate ids and empty hosts are typed config errors; tests with Kafka's examples.

### [To Do] RAFT-0154 Dynamic voters: parse id@host:port:directoryId for --initial-controllers
P0 · M2 · Task · deps: none · ref: raft/src/main/java/org/apache/kafka/raft/DynamicVoter.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/DynamicVoters.java @4.4.0
Done when: DynamicVoters parse and print round-trip, reject duplicate ids, missing directory ids and bad ports with typed errors; tests ported from DynamicVotersTest.

### [To Do] RAFT-0155 Bootstrap checkpoint (0-0) content: KRaftVersionRecord and VotersRecord for standalone and initial controllers
P0 · M2 · Task · deps: RAFT-0154, RAFT-F06 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KRaftControlRecordStateMachine.java @4.4.0; clients/src/main/resources/common/message/VotersRecord.json @4.4.0; clients/src/main/resources/common/message/KRaftVersionRecord.json @4.4.0
Done when: A builder produces the bootstrap snapshot contents (kraft.version 1 plus the voter set) for --standalone (the local controller) and --initial-controllers; SRV-F12 writes the file; reading a bootstrap checkpoint yields the same voters; the bootstrap snapshot is never served by FetchSnapshot.

### [To Do] RAFT-0156 Control-record state machine: kraft.version and voter set history from snapshot and log
P0 · M2 · Task · deps: RAFT-0153, RAFT-0155, RAFT-F05 · ref: raft/src/main/java/org/apache/kafka/raft/internals/KRaftControlRecordStateMachine.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/VoterSetHistory.java @4.4.0; raft/src/main/java/org/apache/kafka/raft/internals/TreeMapLogHistory.java @4.4.0
Done when: Reading the latest snapshot then the log builds a history of (offset, voter set) and (offset, kraft.version), exposing the last (possibly uncommitted) and committed values; truncation drops entries above the truncation offset; static voters apply when no VotersRecord exists; property test against a model.

### [To Do] RAFT-0157 Static versus dynamic voter configuration rules
P0 · M2 · Task · deps: RAFT-0156 · ref: raft/src/main/java/org/apache/kafka/raft/QuorumConfig.java @4.4.0; metadata/src/main/java/org/apache/kafka/controller/metrics/QuorumControllerMetrics.java @4.4.0
Done when: With controller.quorum.voters set and a voter set in the log, the static list is ignored and the IgnoredStaticVoters signal is raised; with neither voters nor bootstrap servers the node refuses to start with a clear error; tests.

### [To Do] RAFT-0158 Cluster id checks on raft requests
P0 · M2 · Task · deps: RAFT-0156, RAFT-F01 · labels: security · ref: raft/src/main/java/org/apache/kafka/raft/KafkaRaftClient.java hasValidClusterId @4.4.0
Done when: Raft requests carrying a cluster id different from the local meta.properties cluster id are rejected with INCONSISTENT_CLUSTER_ID; a null cluster id is accepted (older clients); tests for each raft API.

### [To Do] RAFT-0159 Standalone bootstrap: a single formatted controller becomes leader with kraft.version 1
P0 · M2 · Task · deps: RAFT-0157, RAFT-0158, SRV-F12 · labels: test
Done when: A node formatted with --standalone starts, reads its bootstrap checkpoint, elects itself and writes the start-of-epoch voter set; a restart keeps the same voter set; end-to-end test with the format command.
