FROM golang:alpine as build-env

ENV GO111MODULE=on

RUN apk update && apk add bash ca-certificates git gcc g++ libc-dev

RUN mkdir /chat-app-cli
RUN mkdir -p /chat-app-cli/proto

WORKDIR /chat-app-cli

COPY ./proto/service_grpc.pb.go /chat-app-cli/proto
COPY ./proto/service.pb.go /chat-app-cli/proto

COPY ./.env /chat-app-cli
COPY ./server/main.go /chat-app-cli

COPY go.mod .
COPY go.sum .

RUN go mod download

RUN go build -o chat-app-cli .

CMD ./chat-app-cli