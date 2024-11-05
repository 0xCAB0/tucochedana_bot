#!/usr/bin/bash
docker exec -it postgres psql -U postgres -d tu-coche-dana-bot -c "$1"