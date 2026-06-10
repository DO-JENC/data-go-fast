#!/bin/sh
echo "BACKEND_URL is: $BACKEND_URL"
envsubst '${BACKEND_URL}' < /etc/nginx/conf.d/app.conf.template > /tmp/app.conf
echo "Resolved config:"
exec nginx -g "daemon off;"
