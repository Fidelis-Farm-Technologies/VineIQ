#!/bin/bash
VERSION=`git branch --show-current`

docker build -t fidelismachine/vineiq:${VERSION} -t fidelismachine/vineiq:latest .
