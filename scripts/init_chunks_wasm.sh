#!/bin/sh
WASM=target/wasm32-unknown-unknown/release/chunks.wasm
CONTROLLER=$(dfx canister --network ic id users)
# dfx identity use dapp_box
# IDENTITY=$(dfx identity whoami)

# identity ${IDENTITY} "~/.config/dfx/identity/${IDENTITY}/identity.pem"
ic-repl -r ic << END
import controller = "${CONTROLLER}" as "candid/users.did"
call controller.upload_chunks_wasm(file("${WASM}"))
END
