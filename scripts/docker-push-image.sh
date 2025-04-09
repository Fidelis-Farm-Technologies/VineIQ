#!/bin/bash
VERSION=`git branch --show-current`

#docker push fidelismachine/vineiq:${VERSION} 
docker push fidelismachine/vineiq:latest 
