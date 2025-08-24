#!/usr/bin/env bash
set -x

host=${1:-localhost}

curl -v -d '{}' -H 'Content-Type: application/json' $host:3000/signup

curl -v -d '{"email": "user@example.com"}' -H 'Content-Type: application/json' $host:3000/signup

curl -v -d '{"email": "user@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/signup

curl -v -d '{"email": "user@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
