#!/bin/bash
set -euo pipefail
pkg=stream
templates=$GOPATH/src/github.com/pacedotdev/oto/otohttp/templates
cd "$(dirname "$0")"
rustout=$(realpath ../src)
goout=$(realpath ../pkg/stream)

# Go templates
rm $goout/*.gen.go || true
oto -template $templates/server.go.plush \
    -pkg $pkg \
    -out server.gen.go \
    -ignore Ignorer \
    ./definitions.go
gofmt -w server.gen.go
mv server.gen.go $goout

oto -template $templates/client.go.plush \
    -pkg $pkg \
    -ignore Ignorer \
    -out client.gen.go \
    ./definitions.go
gofmt -w client.gen.go
mv client.gen.go $goout

echo "Successfully generated Go interfaces"

# Rust templates
rm $rustout/types.rs || true
rm $rustout/server.rs || true
rm $rustout/client.rs || true

oto -template $templates/rust/types.rs.plush \
    -pkg $pkg \
    -out types.rs \
    -ignore Ignorer \
    ./definitions.go
rustfmt types.rs
mv types.rs $rustout

oto -template $templates/rust/async_client.rs.plush \
    -pkg $pkg \
    -out client.rs \
    -ignore Ignorer \
    ./definitions.go
rustfmt client.rs
mv client.rs $rustout

oto -template $templates/rust/server_actixweb.rs.plush \
    -pkg $pkg \
    -out server.rs \
    -ignore Ignorer \
    ./definitions.go
rustfmt server.rs
mv server.rs $rustout
echo "Successfully generated Rust interfaces"

echo "Done generating oto interfaces"