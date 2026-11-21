#!/usr/bin/env python3
"""Backlog tool: validate the backlog, pick the next task, print statistics.

Usage:
    backlog.py check [--allow-empty-features] [--epic CODE ...]
    backlog.py next [--all] [-n N]
    backlog.py stats [--write]
    backlog.py show <ID>

The format is specified in docs/backlog/README.md. This script is temporary:
FND-F07 ports it to `cargo xtask backlog`.
"""
import argparse
import os
import re
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
BACKLOG = os.path.dirname(HERE)
README = os.path.join(BACKLOG, "README.md")

STATUSES = ["To Do", "In Progress", "In Review", "Blocked", "Done", "Won't Do"]
CLOSED = {"Done", "Won't Do"}
PRIORITIES = ["P0", "P1", "P2", "P3"]
MILESTONES = [f"M{i}" for i in range(19)]
TASK_TYPES = {"Task", "Spike", "Story", "Bug"}
TRAILERS = ("Owner:", "PR:", "Blocked:", "Won't do:", "Notes:")

HEADING = re.compile(r"^(##|###) \[([^\]]+)\] (\S+) (.+)$")
META = re.compile(r"^(P\d) · (M\d+) · (\w+) · deps: ([^·]+?)(?: · labels: ([^·]+?))?(?: · ref: (.+))?$")


class Item:
    def __init__(self, **kw):
        self.__dict__.update(kw)
        self.tasks = []

    @property
    def mnum(self):
        return int(self.milestone[1:])


def epic_order():
    order = []
    for line in open(README):
        m = re.match(r"^\| (\d+) \| ([A-Z][A-Z-]*) \| \[([^\]]*)\]\(([^)]+)\)", line)
        if m:
            order.append(m.group(2))
    return order


def check_file_numbers(errors):
    """The NN prefix of each epic file must equal its position in the README epic table."""
    for line in open(README):
        m = re.match(r"^\| (\d+) \| ([A-Z][A-Z-]*) \| \[([^\]]*)\]\(([^)]+)\)", line)
        if m and not m.group(4).startswith(f"{int(m.group(1)):02d}-"):
            errors.append(f"README.md: epic {m.group(2)} is #{m.group(1)} but links to {m.group(4)}")
        if m and not os.path.exists(os.path.join(BACKLOG, m.group(4))):
            errors.append(f"README.md: epic table links to missing file {m.group(4)}")


def parse():
    items, errors = {}, []
    files = sorted(f for f in os.listdir(BACKLOG) if f.endswith(".md") and f != "README.md")
    for name in files:
        path = os.path.join(BACKLOG, name)
        lines = open(path).read().split("\n")
        m = re.match(r"^# ([A-Z][A-Z-]*): ", lines[0]) if lines else None
        if not m:
            errors.append(f"{name}:1: first line must be '# <EPIC>: <title>'")
            continue
        code = m.group(1)
        if not re.fullmatch(rf"\d{{2}}-{re.escape(code.lower())}-[a-z0-9-]+\.md", name):
            errors.append(f"{name}: file name must be '<NN>-{code.lower()}-<slug>.md'")
        feature = None
        i = 0
        while i < len(lines):
            line = lines[i]
            if line.startswith("## ") or line.startswith("### "):
                h = HEADING.match(line)
                loc = f"{name}:{i + 1}"
                if not h:
                    errors.append(f"{loc}: malformed heading: {line}")
                    i += 1
                    continue
                level, status, iid, title = h.groups()
                meta_line = lines[i + 1] if i + 1 < len(lines) else ""
                mm = META.match(meta_line)
                if not mm:
                    errors.append(f"{loc}: malformed metadata line after {iid}: {meta_line!r}")
                    i += 1
                    continue
                prio, ms, typ, deps, labels, ref = mm.groups()
                deps = [] if deps.strip() == "none" else [d.strip() for d in deps.split(",")]
                it = Item(id=iid, level=level, status=status, title=title, priority=prio, milestone=ms,
                          type=typ, deps=deps, labels=(labels or "").strip(), ref=(ref or "").strip(),
                          file=name, line=i + 1, epic=code, feature=None, done_when=None, trailers={})
                if status not in STATUSES:
                    errors.append(f"{loc}: {iid}: unknown status '{status}'")
                if prio not in PRIORITIES:
                    errors.append(f"{loc}: {iid}: unknown priority '{prio}'")
                if ms not in MILESTONES:
                    errors.append(f"{loc}: {iid}: unknown milestone '{ms}'")
                if any(not d for d in deps):
                    errors.append(f"{loc}: {iid}: empty dependency entry")
                j = i + 2
                if level == "##":
                    if not re.fullmatch(rf"{re.escape(code)}-F\d{{2,3}}", iid):
                        errors.append(f"{loc}: feature id '{iid}' must be {code}-Fnn")
                    if typ != "Feature":
                        errors.append(f"{loc}: {iid}: '##' items must have type Feature")
                    if j >= len(lines) or not lines[j].startswith("Scope: "):
                        errors.append(f"{loc}: {iid}: missing 'Scope:' line")
                    feature = it
                else:
                    if not re.fullmatch(rf"{re.escape(code)}-\d{{4}}", iid):
                        errors.append(f"{loc}: task id '{iid}' must be {code}-nnnn")
                    if typ not in TASK_TYPES:
                        errors.append(f"{loc}: {iid}: task type must be one of {sorted(TASK_TYPES)}")
                    if j < len(lines) and lines[j].startswith("Done when: ") and len(lines[j]) > 20:
                        it.done_when = lines[j][len("Done when: "):]
                        j += 1
                    else:
                        errors.append(f"{loc}: {iid}: missing or empty 'Done when:' line")
                    while j < len(lines) and lines[j].startswith(TRAILERS):
                        key = lines[j].split(":", 1)[0]
                        it.trailers[key] = lines[j].split(":", 1)[1].strip()
                        j += 1
                    if feature is None:
                        errors.append(f"{loc}: task {iid} appears before any feature")
                    else:
                        it.feature = feature.id
                        feature.tasks.append(it)
                    if status == "Blocked" and "Blocked" not in it.trailers:
                        errors.append(f"{loc}: {iid}: Blocked tasks need a 'Blocked:' line")
                    if status == "Won't Do" and "Won't do" not in it.trailers:
                        errors.append(f"{loc}: {iid}: Won't Do tasks need a \"Won't do:\" line")
                if iid in items:
                    errors.append(f"{loc}: duplicate id {iid} (also {items[iid].file}:{items[iid].line})")
                items[iid] = it
                i = j
                continue
            i += 1
    return items, errors


def tasks_of(items):
    return [it for it in items.values() if it.level == "###"]


def effective_deps(it, items):
    """Task ids a task waits on, expanding feature dependencies per the README rules."""
    out = set()
    deps = list(it.deps)
    if it.level == "###" and it.feature:
        deps += items[it.feature].deps
    for d in deps:
        dep = items.get(d)
        if dep is None:
            continue
        if dep.level == "##":
            out.update(t.id for t in dep.tasks if t.mnum <= it.mnum and t.id != it.id)
        else:
            out.add(d)
    return out


def check(items, errors, allow_empty):
    warnings = []
    for it in items.values():
        loc = f"{it.file}:{it.line}"
        for d in it.deps:
            dep = items.get(d)
            if dep is None:
                errors.append(f"{loc}: {it.id}: unknown dependency {d}")
                continue
            if d == it.id:
                errors.append(f"{loc}: {it.id}: depends on itself")
            if dep.epic != it.epic and dep.level != "##":
                errors.append(f"{loc}: {it.id}: cross-epic dependency {d} must be a feature id")
            if dep.level == "###" and dep.mnum > it.mnum:
                errors.append(f"{loc}: {it.id} ({it.milestone}) depends on {d} in later milestone {dep.milestone}")
            if dep.level == "##" and dep.tasks and it.level == "###" and not any(t.mnum <= it.mnum for t in dep.tasks):
                warnings.append(f"{loc}: {it.id} ({it.milestone}) depends on feature {d}, which has no tasks at or before {it.milestone}")
        if it.level == "##":
            if not it.tasks:
                if not allow_empty:
                    errors.append(f"{loc}: feature {it.id} has no tasks")
                continue
            first = min(t.mnum for t in it.tasks)
            if it.mnum != first:
                errors.append(f"{loc}: feature {it.id} milestone {it.milestone} must equal its earliest task milestone M{first}")
            statuses = {t.status for t in it.tasks}
            if statuses <= CLOSED:
                want = "Done"
            elif statuses & {"In Progress", "In Review", "Done", "Blocked"}:
                want = "In Progress"
            else:
                want = "To Do"
            if it.status != want:
                errors.append(f"{loc}: feature {it.id} status should be '{want}' given its tasks")
    graph = {t.id: effective_deps(t, items) for t in tasks_of(items)}
    state = {}
    for start in graph:
        if start in state:
            continue
        stack = [(start, iter(sorted(graph[start])))]
        state[start] = 1
        path = [start]
        while stack:
            node, it_ = stack[-1]
            nxt = next(it_, None)
            if nxt is None:
                state[node] = 2
                stack.pop()
                path.pop()
                continue
            if nxt not in graph:
                continue
            if state.get(nxt) == 1:
                cyc = path[path.index(nxt):] + [nxt]
                errors.append("dependency cycle: " + " -> ".join(cyc))
                continue
            if nxt not in state:
                state[nxt] = 1
                stack.append((nxt, iter(sorted(graph[nxt]))))
                path.append(nxt)
    return warnings


def ready_tasks(items):
    tasks = tasks_of(items)
    open_ms = sorted({t.mnum for t in tasks if t.status not in CLOSED})
    ready = [t for t in tasks if t.status == "To Do"
             and all(items[d].status in CLOSED for d in effective_deps(t, items))]
    return (open_ms[0] if open_ms else None), ready


def sort_key(order):
    rank = {c: i for i, c in enumerate(order)}
    return lambda t: (t.mnum, PRIORITIES.index(t.priority), rank.get(t.epic, 999), t.id)


def cmd_next(items, args):
    current, ready = ready_tasks(items)
    if current is None:
        print("Everything is Done or Won't Do.")
        return
    ready.sort(key=sort_key(epic_order()))
    if not args.all:
        in_current = [t for t in ready if t.mnum == current]
        ready = (in_current or ready)[: args.n]
    print(f"Current milestone: M{current}")
    for t in ready:
        print(f"{t.id:16} {t.priority} {t.milestone:4} {t.type:6} {t.title}")
        print(f"{'':16} {t.file}:{t.line} · Done when: {t.done_when}")
    if not ready:
        print("No ready tasks: everything open is blocked or claimed.")


def stats_text(items):
    order = epic_order()
    tasks = tasks_of(items)
    by_epic = defaultdict(list)
    for t in tasks:
        by_epic[t.epic].append(t)
    feats = defaultdict(int)
    for it in items.values():
        if it.level == "##":
            feats[it.epic] += 1
    lines = ["| Epic | Features | Tasks | Done | Open | Highest task id |", "|---|---|---|---|---|---|"]
    for code in order:
        ts = by_epic.get(code, [])
        done = sum(t.status in CLOSED for t in ts)
        top = max((t.id for t in ts), default="—")
        lines.append(f"| {code} | {feats[code]} | {len(ts)} | {done} | {len(ts) - done} | {top} |")
    done = sum(t.status in CLOSED for t in tasks)
    lines.append(f"| **Total** | **{sum(feats.values())}** | **{len(tasks)}** | **{done}** | **{len(tasks) - done}** | |")
    lines += ["", "| Milestone | Tasks | Done | Open |", "|---|---|---|---|"]
    for m in MILESTONES:
        ts = [t for t in tasks if t.milestone == m]
        d = sum(t.status in CLOSED for t in ts)
        lines.append(f"| {m} | {len(ts)} | {d} | {len(ts) - d} |")
    return "\n".join(lines)


STATS_RE = re.compile(r"(<!-- backlog-stats:start -->\n)(.*?)(<!-- backlog-stats:end -->)", re.S)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check")
    c.add_argument("--allow-empty-features", action="store_true",
                   help="construction mode: allow features without tasks and skip the totals check")
    c.add_argument("--epic", action="append", default=[],
                   help="only report problems in these epics (repeatable); everything is still parsed")
    n = sub.add_parser("next")
    n.add_argument("--all", action="store_true")
    n.add_argument("-n", type=int, default=10)
    s = sub.add_parser("stats")
    s.add_argument("--write", action="store_true")
    sh = sub.add_parser("show")
    sh.add_argument("id")
    args = ap.parse_args()

    items, errors = parse()
    if args.cmd == "check":
        warnings = check(items, errors, args.allow_empty_features)
        check_file_numbers(errors)
        readme = open(README).read()
        m = STATS_RE.search(readme)
        if not m:
            errors.append("README.md: missing backlog-stats markers")
        elif not args.allow_empty_features and tasks_of(items) and m.group(2).strip() != stats_text(items).strip():
            errors.append("README.md: totals are out of date; run `backlog.py stats --write`")
        missing = {it.epic for it in items.values()} - set(epic_order())
        if missing:
            errors.append(f"README.md: epics missing from the epic table: {sorted(missing)}")
        if args.epic:
            # An epic's file names contain `-<code>-` (`02-fnd-foundation.md`) and its ids `<CODE>-`.
            mine = tuple(f"-{c.lower()}-" for c in args.epic)
            ids = tuple(f"{c}-" for c in args.epic)
            keep = lambda msg: any(m in msg for m in mine) or any(i in msg for i in ids)
            errors[:] = [e for e in errors if keep(e)]
            warnings = [w for w in warnings if keep(w)]
        for w in warnings:
            print("warning:", w)
        for e in errors:
            print("error:", e)
        print(f"{len(items)} items ({len(tasks_of(items))} tasks), {len(errors)} errors, {len(warnings)} warnings")
        sys.exit(1 if errors else 0)
    if errors:
        for e in errors[:20]:
            print("error:", e)
        sys.exit("backlog has format errors; run `backlog.py check`")
    if args.cmd == "next":
        cmd_next(items, args)
    elif args.cmd == "stats":
        text = stats_text(items)
        if args.write:
            readme = open(README).read()
            readme = STATS_RE.sub(lambda m: m.group(1) + text + "\n" + m.group(3), readme)
            open(README, "w").write(readme)
        print(text)
    elif args.cmd == "show":
        it = items.get(args.id)
        if not it:
            sys.exit(f"unknown id {args.id}")
        print(f"{it.id} [{it.status}] {it.title}\n  {it.priority} · {it.milestone} · {it.type} · {it.file}:{it.line}")
        if it.done_when:
            print(f"  Done when: {it.done_when}")
        for d in (sorted(effective_deps(it, items)) if it.level == "###" else it.deps):
            print(f"  needs {d} [{items[d].status}] {items[d].title}")
        for other in items.values():
            if it.id in other.deps:
                print(f"  needed by {other.id} [{other.status}] {other.title}")


if __name__ == "__main__":
    main()
