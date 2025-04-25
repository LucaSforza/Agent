#!/bin/sh

if [ -d $1 ]; then
    echo "Passare come argomento l'esempio da eseguire"
    echo "Quelli disponibili: n_queen protein_folding"
    exit 1
fi

EXAMPLE=$1
shift

cargo run --package agent --example $EXAMPLE --release -- $@