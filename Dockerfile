ARG PROJECT_NAME=technical-commercial-proposal
ARG RUST_VERSION=1.57.0
ARG ASTRA_VERSION=1.7.5
ARG ASTRA_IMAGE="docker-asez.rc.inlinegroup.ru/cicd/astra-linux-1-7-5-rust-1.57.0-docker-dockercompose:latest"
ARG BASE_SYSTEM="ASTRA Linux 1.7.5"
ARG AUTHOR="Nikolay Galko, Ibragim Kusov"

###################################
#FROM ${ASTRA_IMAGE}:${ASTRA_VERSION} AS builder
FROM ${ASTRA_IMAGE}:latest AS builder
ARG PROJECT_NAME
ARG RUST_VERSION
ARG ASTRA_VERSION
ARG ASTRA_IMAGE
ARG BASE_SYSTEM
ARG AUTHOR
ARG GIT_COMMIT_ID

LABEL DESTINATION="[Destination] Source code for service: ${PROJECT_NAME}"
LABEL AUTHOR="$AUTHOR"
LABEL BASE_SYSTEM="$BASE_SYSTEM"
LABEL RUST_VERSION="$RUST_VERSION"
LABEL GIT_COMMIT_ID="$GIT_COMMIT_ID"

WORKDIR /usr/src/app

COPY . .

RUN ((mkdir /opt/app) && ( \
    cargo build -p $PROJECT_NAME  \
    --target x86_64-unknown-linux-gnu  \
    --release --locked) &&  \
    (cp /usr/src/app/target/x86_64-unknown-linux-gnu/release/"$PROJECT_NAME" /opt/app/app-before-opt))


# Оптимизируем размер бинарника через binutils (upx у нас нет в контуре заказчика)
RUN (strip /opt/app/app-before-opt -o /opt/app/app-striped) && (cp /opt/app/app-striped /opt/app/app)

###################################
# * --- Running Stage ---
#FROM scratch
FROM docker-asez.rc.inlinegroup.ru/cicd/astra-linux:1.7.5-curl
#FROM ${ASTRA_IMAGE}:${ASTRA_VERSION}
ARG PROJECT_NAME
ARG RUST_VERSION
ARG ASTRA_VERSION
ARG ASTRA_IMAGE
ARG BASE_SYSTEM
ARG AUTHOR
ARG GIT_COMMIT_ID
ARG BUILD_VERSION_ID

LABEL DESTINATION="[Destination] Runner for service: ${PROJECT_NAME}"
LABEL AUTHOR="$AUTHOR"
LABEL RUST_VERSION="$RUST_VERSION"
LABEL VERSION_ID=$BUILD_VERSION_ID
LABEL ASTRA_IMAGE="$ASTRA_IMAGE"
LABEL ASTRA_VERSION="$ASTRA_VERSION"
LABEL GIT_COMMIT_ID="$GIT_COMMIT_ID"
ENV DEV_COMMIT_ID="https://rcgitlab.inlinegroup.ru/workspace/monorepo/-/commit/$GIT_COMMIT_ID"
ENV VERSION_ID=$BUILD_VERSION_ID
WORKDIR /opt/app
COPY --from=builder /opt/app/app /opt/app/app
EXPOSE 3000

CMD ["/opt/app/app"]
