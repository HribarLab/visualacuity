#!/bin/zsh

set -e

# Fussing with zsh profile problems
eval "$(pyenv init -)"
pyenv activate $(cat .python-version)
###


PYTHON_SRC=src/visualacuity-python

unset CONDA_PREFIX
python -m maturin develop --manifest-path ${PYTHON_SRC}/Cargo.toml
