#!/bin/sh
tmux renamew -t $TMX_WINID building...
clear
if exectime cargo build --release; then
cp target/release/lib{{name}}.so ../godot/lib/lib{{name}}.so
fi

