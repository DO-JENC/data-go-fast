#!/bin/sh
envsubst '${VITE_API_URL}' < /etc/nginx/conf.d/app.conf.template > /etc/nginx/conf.d/app.conf
exec nginx -g "daemon off;"
