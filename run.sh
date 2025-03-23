#!/bin/sh

if [ -d $1 ]; then
    echo "Passare come argomento l'esempio da eseguire"
    echo "Quelli disponibili: n_queen"
    exit 1
fi

cargo run --package agent --example $1 --release