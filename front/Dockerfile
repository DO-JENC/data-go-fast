#######################
# Dependencies
#######################

FROM node:24-slim AS deps
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app
COPY front/package.json front/pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install --frozen-lockfile --ignore-scripts

#######################
# Build
#######################

FROM deps AS builder
# Copy source AFTER deps so the node_modules layer is cached
COPY front/ .
RUN pnpm run build

#######################
# Production
#######################

FROM nginx:1.27-alpine AS runner
# Drop root: run nginx as non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
# Harden nginx: remove default config, use custom minimal one
RUN rm /etc/nginx/conf.d/default.conf
COPY --chown=appuser:appgroup front/nginx-main.conf /etc/nginx/nginx.conf
COPY --chown=appuser:appgroup front/nginx.conf /etc/nginx/conf.d/app.conf.template
COPY --chown=appuser:appgroup front/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Copy only the compiled static assets
COPY --from=builder --chown=appuser:appgroup /app/dist /usr/share/nginx/html

# nginx needs to write temp/pid files; give non-root access
RUN mkdir -p /var/cache/nginx /var/run /run \
    && chown -R appuser:appgroup /var/cache/nginx /var/run /run /var/log/nginx \
    && ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \

CMD wget -qO- http://localhost:8080/health || exit 1

CMD ["/docker-entrypoint.sh"]
