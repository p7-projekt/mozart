FROM golang:1.23 AS build
WORKDIR /build
COPY . /build
ENV CGO_ENABLED=0
RUN go build -o /bin/mozart ./main.go

FROM alpine:3.20
COPY --from=build /bin/mozart /bin/mozart
RUN apk add --no-cache "ghc=9.8.2-r1"
EXPOSE 8080
CMD ["/bin/mozart"]

