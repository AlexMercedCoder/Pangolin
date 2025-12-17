# Build Stage
FROM node:20-alpine as builder

WORKDIR /usr/src/pangolin-ui

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build the SvelteKit app
RUN npm run build

# Runtime Stage
FROM node:20-alpine

WORKDIR /app

# Copy built files
COPY --from=builder /usr/src/pangolin-ui/build ./build
COPY --from=builder /usr/src/pangolin-ui/package.json ./package.json
COPY --from=builder /usr/src/pangolin-ui/node_modules ./node_modules

# Expose port (default 3000)
EXPOSE 3000

# Set Node Adapter env if needed, usually just node build/index.js
CMD ["node", "build/index.js"]
