#!/bin/bash
git describe --tags --exact-match 2>/dev/null || git -c "core.abbrev=8" show -s "--format=%cd-%h" "--date=format:%Y%m%d-%H%M%S"
