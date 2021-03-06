#!/bin/bash
# integritycheck - https://github.com/asmuth/integritycheck
# Copyright (c) 2018, Paul Asmuth <paul@asmuth.com>
#
# This file is part of the "integritycheck" project. integritycheck is free software
# licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License.
set -uex

source ./test/test-util.sh
mkdir "${TEST_TMPDIR}/repo"
cd "${TEST_TMPDIR}/repo"

echo "A" > testA
echo "B" > testB
echo "C" > testC

ic init
ic status

echo "X" > testX
echo "C2" > testC

mkdir testDir
touch testDir/1
touch testDir/2
touch testDir/3

if ic status --colours=off > "../status.raw"; then
  echo "exit code must be one"
  exit 1
fi

cat "../status.raw" | grep -vE "^Repository" | grep -vE "^Last Snapshot" > "../status"

(cat > "../status.expected") <<EOF
Total Size: 6B (3 files)
Status: DIRTY

    modified "testC" (metadata modifications only)
    created  "testDir/1"
    created  "testDir/2"
    created  "testDir/3"
    created  "testX"

EOF

cat ../status
diff "../status" "../status.expected"

sleep 0.01

ic ack -y testX testDir

if ic status --colours=off > "../status.raw"; then
  echo "exit code must be one"
  exit 1
fi

cat "../status.raw" | grep -vE "^Repository" | grep -vE "^Last Snapshot" > "../status"

(cat > "../status.expected") <<EOF
Total Size: 8B (7 files)
Status: DIRTY

    modified "testC" (metadata modifications only)

EOF

diff "../status" "../status.expected"

sleep 0.01

ic ack -y testC

ic status # must be clean
