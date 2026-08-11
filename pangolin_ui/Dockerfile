# Build Stage
FROM node:20-alpine AS builder

WORKDIR /usr/src/pangolin-ui

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build the SvelteKit app
RUN npm run build

# Drop the build toolchain from node_modules.
#
# The runtime stage used to copy the *whole* node_modules across, so the
# published image shipped all 22 devDependencies - vite, playwright,
# svelte-check, vitest, the tailwind toolchain - none of which run in
# production. adapter-node needs only the `dependencies` entries alongside its
# bundled output, and this project has exactly one.
RUN npm prune --omit=dev

# Runtime Stage
FROM node:20-alpine

LABEL org.opencontainers.image.title="Pangolin UI" \
      org.opencontainers.image.description="Web interface for the Pangolin Iceberg catalog" \
      org.opencontainers.image.source="https://github.com/AlexMercedCoder/pangolin" \
      org.opencontainers.image.licenses="MIT"

WORKDIR /app

# Copy built files
COPY --from=builder --chown=node:node /usr/src/pangolin-ui/build ./build
COPY --from=builder --chown=node:node /usr/src/pangolin-ui/package.json ./package.json
COPY --from=builder --chown=node:node /usr/src/pangolin-ui/node_modules ./node_modules

# The node image ships an unprivileged `node` user; the image had no USER
# directive at all, so the server ran as root.
USER node

# Expose port (default 3000)
EXPOSE 3000

# Set Node Adapter env if needed, usually just node build/index.js
CMD ["node", "build/index.js"]
