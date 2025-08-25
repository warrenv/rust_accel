#!/usr/bin/env bash
set -x

host=${1:-localhost}

curl -v -d '{}' -H 'Content-Type: application/json' $host:3000/signup
echo

curl -v -d '{"email": "user@example.com"}' -H 'Content-Type: application/json' $host:3000/signup
echo

curl -v -d '{"email": "user@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/signup
echo

curl -v -d '{"email": "user@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
echo

# invalid inputs = 400
curl -v -d '{"email": "", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
echo
curl -v -d '{"email": "user@example.com", "password": "p123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
echo
curl -v -d '{"email": "userexample.com", "password": "p123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
echo

# incorrect inputs = 401
curl -v -d '{"email": "user2@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
echo
curl -v -d '{"email": "user2@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
echo
curl -v -d '{"email": "USER2@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
echo
curl -v -d '{"email": "user2@example.com", "password": "PASSWORD123"}' -H 'Content-Type: application/json' $host:3000/login
echo

# correct inputs = 200
curl -v -d '{"email": "user3@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
echo
curl -v -d '{"email": "user3@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
echo

# logout tests
#curl -v -d '{"email": "user4@example.com", "password": "password123", "requires2FA": false}' -H 'Content-Type: application/json' $host:3000/signup
curl -v  --cookie-jar curl-cookies.txt -d '{"email": "user4@example.com", "password": "password123"}' -H 'Content-Type: application/json' $host:3000/login
echo
cat curl-cookies.txt
echo
curl -v --cookie-jar curl-cookies.txt --cookie curl-cookies.txt -X POST -H 'Content-Type: application/json' $host:3000/logout
echo
cat curl-cookies.txt
echo
curl -v --cookie-jar curl-cookies.txt --cookie curl-cookies.txt -X POST -H 'Content-Type: application/json' $host:3000/logout
echo
cat curl-cookies.txt
echo
rm curl-cookies.txt
