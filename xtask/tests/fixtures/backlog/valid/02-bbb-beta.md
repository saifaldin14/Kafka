# BBB: Beta

## [To Do] BBB-F01 Feature depending on another epic
P0 · M0 · Feature · deps: AAA-F01
Scope: Its M0 task waits only for AAA-F01's tasks up to M0.

### [To Do] BBB-0001 A task whose feature dependency is satisfied up to its milestone
P0 · M0 · Story · deps: none
Done when: It is ready although AAA-0005 is open.

### [Won't Do] BBB-0002 A dropped task
P1 · M2 · Task · deps: none
Done when: Never; it was split instead.
Won't do: split into BBB-0003.

### [To Do] BBB-0003 A task waiting for a task in the same epic
P1 · M2 · Task · deps: BBB-0001
Done when: It runs after BBB-0001.
