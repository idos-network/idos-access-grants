FROM debian

RUN apt update && apt install -y \
    curl \
    jq \
    npm \
&& rm -rf /var/lib/apt/lists/*

RUN npm install --global yarn

RUN adduser runner && adduser runner sudo && mkdir ~runner/code

WORKDIR /home/runner/code

COPY . .
