#!/bin/bash
DIR="$( cd "$( dirname "$0" )" && pwd )"
cd "$DIR"
cd ../docs

source venv/bin/activate
mkdocs serve
