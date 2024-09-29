#!/usr/bin/env sh
set -euo pipefail
SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
PROJECT_DIR=$(readlink -f "$SCRIPT_DIR/../..")

TEMP_DIR=$(mktemp -t -d XXXX-highlightjs)
echo $TEMP_DIR
git clone https://github.com/highlightjs/highlight.js.git $TEMP_DIR
cp $SCRIPT_DIR/stiltshtml.js $TEMP_DIR/src/languages/

pushd $TEMP_DIR
npm i
node tools/build.js -n rust ini shell :web
popd

cp $TEMP_DIR/build/highlight.js $PROJECT_DIR/book/theme/highlight.js

rm -r --interactive=never -- $TEMP_DIR
