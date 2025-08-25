#!/usr/bin/env bash
set -x

host=${1:-localhost}

curl -v -d '{}' -H 'Content-Type: application/json' $host:3000/signup

curl -v -d '{"email": "user@example.com"}' -H 'Content-Type: application/json' $host:3000/signup

curl -v -d '{"email": "user@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/signup

curl -v -d '{"email": "user@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup

# invalid inputs = 400
curl -v -d '{"email": "", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
curl -v -d '{"email": "user@example.com", "password": "p123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
curl -v -d '{"email": "userexample.com", "password": "p123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup

# incorrect inputs = 401
curl -v -d '{"email": "user2@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
curl -v -d '{"email": "user2@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
curl -v -d '{"email": "USER2@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
curl -v -d '{"email": "user2@example.com", "password": "PASSWORD123"}' -H 'Content-Type: application/json' $host:3000/login

# correct inputs = 200
curl -v -d '{"email": "user3@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
curl -v -d '{"email": "user3@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
