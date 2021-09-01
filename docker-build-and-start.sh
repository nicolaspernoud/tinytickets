#!/bin/bash

WD="$(
    cd "$(dirname "$0")"
    pwd -P
)"

docker stop tinytickets
docker rm tinytickets
docker build . -t tinytickets
docker run -it \
    --name tinytickets \
    -p 8000:8000 \
    -v ${WD}/db:/app/db \
    -v ${WD}/data:/app/data \
    -e MAIL_SERVER=mail.***.fr \
    -e MAIL_USER=***@***.fr \
    -e MAIL_PASSWORD="***" \
    -e MAIL_TO=***.***@***.com \
    -e MAIL_FROM=***.***@***.com \
    -e ADMIN_TOKEN=admin \
    -e USER_TOKEN=user \
    -e DESK_TOKEN=desk \
    tinytickets
