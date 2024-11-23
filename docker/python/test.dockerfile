FROM --platform=linux/amd64 rust:alpine3.20

RUN apk add --no-cache \
    python3=3.12.7-r0 \
    musl-dev \
    shadow \
    acl
RUN mkdir /mozart
RUN useradd -M -N restricted # -M means no home folder, -N means no user group
RUN rustup target add x86_64-unknown-linux-musl

RUN setfacl -m u:restricted:r-x /mozart 
RUN setfacl -m u:restricted:r-x /usr

RUN setfacl -m u:restricted:--- / 
RUN setfacl -m u:restricted:--- /tmp
RUN setfacl -m u:restricted:--- /var/tmp
RUN setfacl -m u:restricted:--- /dev/shm
RUN setfacl -m u:restricted:--- /var/spool/mail
RUN setfacl -m u:restricted:--- /var/mail
RUN setfacl -m u:restricted:--- /var/cache
RUN setfacl -m u:restricted:--- /var/log

WORKDIR /test
COPY . .
ENV PATH="$PATH:/usr/bin/python"
CMD ["cargo", "test", "--target=x86_64-unknown-linux-musl", "--features", "python"]
