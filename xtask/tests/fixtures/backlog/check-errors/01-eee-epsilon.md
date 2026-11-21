# EEE: Epsilon

## [To Do] EEE-F01 Feature with the wrong milestone and status
P0 · M1 · Feature · deps: none
Scope: Its earliest task is in M0 and one of its tasks is Done.

### [Done] EEE-0001 Unknown and self dependencies
P0 · M0 · Task · deps: EEE-9999, EEE-0001
Done when: Both dependencies are reported.

### [To Do] EEE-0002 Depends on a task in a later milestone
P0 · M0 · Task · deps: EEE-0003
Done when: The later-milestone dependency is reported.

### [To Do] EEE-0003 First task of a cycle
P0 · M2 · Task · deps: EEE-0004
Done when: The cycle is reported.

### [To Do] EEE-0004 Second task of a cycle
P0 · M2 · Task · deps: EEE-0003
Done when: The cycle is reported.

## [To Do] EEE-F02 Feature without tasks
P1 · M0 · Feature · deps: none
Scope: Reported unless empty features are allowed.
