#!/usr/bin/env bash
set -e

openssl req -x509 -newkey rsa:4096 -days 3650 -nodes \
	-keyout ca.key \
	-out ca.crt \
	-subj "/C=CN/ST=Dev/L=Dev/O=LocalCA/OU=RootCA/CN=Local Root CA"

openssl req -newkey rsa:4096 -nodes \
	-keyout server.key \
	-out server.csr \
	-subj "/C=CN/ST=Dev/L=Dev/O=LocalServer/OU=Server/CN=localhost"

openssl x509 -req -in server.csr \
	-CA ca.crt -CAkey ca.key -CAcreateserial \
	-out server.crt -days 3650 -sha256

echo "DONE"
