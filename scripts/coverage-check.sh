#!/usr/bin/env bash
#
# Fast-suite coverage gate (ADR-0013).
#
# `cargo llvm-cov` reports line coverage but, on stable Rust, has no way to
# ignore an individual line. This script applies the repo's line markers to the
# LCOV report and then fails when any remaining measured line is uncovered.
#
#   // coverage:ignore: reason           ignore this line
#   // coverage:ignore-start: reason     ignore from here
#   // coverage:ignore-end              ... to here
#
# A marker needs a reason after `coverage:ignore:`; markers are only for lines
# that genuinely cannot be exercised by the fast suite. Code covered by the
# component or acceptance suites is excluded by filename in the `just coverage`
# recipe instead.
#
# Usage: coverage-check.sh [--report|--check] [lcov-file]
#   --report  filter the LCOV report in place (coverage recipe)
#   --check   fail if any measured line is still uncovered (coverage-check)

set -euo pipefail

mode="--report"
if [[ $# -gt 0 && $1 == --* ]]; then
    mode=$1
    shift
fi
lcov_file=${1:-lcov.info}

if [[ $mode != --report && $mode != --check ]]; then
    echo "coverage-check: unknown mode '$mode' (expected --report or --check)" >&2
    exit 2
fi
if [[ ! -f $lcov_file ]]; then
    echo "coverage-check: '$lcov_file' does not exist; run 'just coverage' first" >&2
    exit 2
fi

tmp=$(mktemp "${TMPDIR:-/tmp}/coverage-check.XXXXXX")
trap 'rm -f "$tmp"' EXIT

set +e
awk -v mode="$mode" '
BEGIN {
    ignored_lines = 0
    uncovered = 0
}

function fail(message) {
    print "coverage-check: " message > "/dev/stderr"
    bad = 1
}

function flush(    i, line, parts, lf, lh) {
    if (!section) {
        return
    }

    lf = 0
    lh = 0
    for (i = 1; i <= records; i++) {
        line = record[i]
        if (line !~ /^DA:/) {
            continue
        }
        split(substr(line, 4), parts, ",")
        if (ignored[parts[1] + 0]) {
            ignored_lines++
            continue
        }
        lf++
        if (parts[2] + 0 > 0) {
            lh++
        } else {
            print "uncovered: " source_path ":" parts[1] > "/dev/stderr"
        }
    }

    for (i = 1; i <= records; i++) {
        line = record[i]
        if (line ~ /^LF:/ || line ~ /^LH:/) {
            continue
        }
        if (line ~ /^DA:/) {
            split(substr(line, 4), parts, ",")
            if (ignored[parts[1] + 0]) {
                continue
            }
        }
        print line
    }
    print "LF:" lf
    print "LH:" lh
    print "end_of_record"

    if (lh < lf) {
        uncovered += lf - lh
    }
    section = 0
}

function load_source(path,    i, n, line, in_ignore) {
    delete ignored
    delete source
    in_ignore = 0
    i = 0
    while ((getline line < path) > 0) {
        sub(/\r$/, "", line)
        source[++i] = line
    }
    close(path)
    if (i == 0) {
        fail("cannot read source file '" path "'")
        return
    }
    n = i

    for (i = 1; i <= n; i++) {
        line = source[i]
        if (line ~ /coverage:ignore-start/) {
            if (line !~ /coverage:ignore-start:[[:space:]]*[^[:space:]]/) {
                fail("coverage:ignore-start needs a reason in '" path "'")
            }
            ignored[i] = 1
            in_ignore = 1
            continue
        }
        if (line ~ /coverage:ignore-end/) {
            ignored[i] = 1
            in_ignore = 0
            continue
        }
        if (in_ignore) {
            ignored[i] = 1
            continue
        }
        if (line ~ /coverage:ignore:/) {
            if (line !~ /coverage:ignore:[[:space:]]*[^[:space:]]/) {
                fail("coverage:ignore needs a reason in '" path "'")
            }
            ignored[i] = 1
        }
    }
    if (in_ignore) {
        fail("unterminated coverage:ignore-start in '" path "'")
    }
}

/^SF:/ {
    if (section) {
        flush()
    }
    section = 1
    source_path = substr($0, 4)
    records = 0
    record[++records] = $0
    load_source(source_path)
    next
}

{
    if (!section) {
        print
        next
    }
    if ($0 == "end_of_record") {
        flush()
        next
    }
    record[++records] = $0
}

END {
    if (section) {
        flush()
    }
    print "coverage-check: " ignored_lines " ignored line(s), " uncovered " uncovered line(s)" > "/dev/stderr"
    if (bad) {
        exit 2
    }
    if (mode == "--check" && uncovered > 0) {
        exit 1
    }
}
' "$lcov_file" > "$tmp"
status=$?
set -e

if [[ $mode == --report && $status -eq 0 ]]; then
    mv "$tmp" "$lcov_file"
fi

exit "$status"
