POST http://localhost:8000/unseal
X-Chamber-Key: 111

HTTP 200
[Asserts]

POST http://localhost:8000/login
Content-Type: application/json
{
    "username": "root",
    "password": "this"
}

HTTP 200

[Captures]
access_token: jsonpath "$['access_token']"

POST http://localhost:8000/secrets/set
Content-Type: application/json
Authorization: Bearer {{access_token}}
{
    "key":"hello",
    "value":"world",
    "tags": ["shuttle"]
}

HTTP 201

POST http://localhost:8000/secrets/set
Content-Type: application/json
Authorization: Bearer {{access_token}}
{
    "key":"made_by",
    "value":"shuttle",
    "tags": ["shuttle"]
}

HTTP 201

POST http://localhost:8000/secrets/by_tag
Content-Type: application/json
Authorization: Bearer {{access_token}}
{
    "key": "shuttle"
}

HTTP 200
[Asserts]
jsonpath "$[0].key" == "hello"
jsonpath "$[0].value" == "world"
jsonpath "$[1].key" == "made_by"
jsonpath "$[1].value" == "shuttle"
