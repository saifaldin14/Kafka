# AAA: Alpha

An epic whose features cover every task status.

## [In Progress] AAA-F01 First feature
P0 · M0 · Feature · deps: none
Scope: One finished task and one later-milestone task.

### [Done] AAA-0001 A finished task
P0 · M0 · Task · deps: none · labels: docs, test · ref: ADR-0001
Done when: The task is finished and verified.
PR: #1

### [To Do] AAA-0005 A later-milestone task of the first feature
P1 · M2 · Task · deps: none
Done when: It is ready but in a later milestone.

## [In Progress] AAA-F02 Second feature
P1 · M0 · Feature · deps: AAA-F01
Scope: Ready, waiting, claimed and blocked tasks.

### [To Do] AAA-0002 A ready task
P1 · M0 · Task · deps: AAA-0001
Done when: It appears in `next`.

### [To Do] AAA-0003 A task waiting for a claimed task
P0 · M0 · Task · deps: AAA-0004
Done when: It waits for AAA-0004.

### [In Progress] AAA-0004 A claimed spike
P2 · M0 · Spike · deps: none · labels: research · ref: KIP-1 · section 3
Done when: An ADR is accepted.
Owner: someone
Notes: Claimed tasks are skipped by `next`.

### [Blocked] AAA-0006 A blocked bug
P3 · M1 · Bug · deps: none · labels: security
Done when: The bug is fixed.
Blocked: waiting for a decision.
