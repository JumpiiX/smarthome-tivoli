FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies including Chrome
RUN apt-get update && apt-get install -y \
    wget \
    gnupg \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libdrm2 \
    libxkbcommon0 \
    libxss1 \
    libnss3 \
    libgtk-3-0 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Install Chrome
RUN wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -s /bin/bash app

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/knx-homekit-bridge ./knx-homekit-bridge

# Copy config files if they exist
COPY device_mappings.toml ./

# Create chrome_data directory with proper permissions
RUN mkdir -p chrome_data && chown app:app chrome_data

# Switch to app user
USER app

# Expose the API port
EXPOSE 8080

# Run the application in main mode with headless Chrome
CMD ["./knx-homekit-bridge", "--headless"]
