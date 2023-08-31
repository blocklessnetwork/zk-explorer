# ZK Explorer

Proof Generation API

`/api/prove`

**WASM**

Simple Factor
```bash
curl -X POST "http://localhost:3000/api/prove" -H "Content-Type: application/json" -d '{
  "cid": "bafkreidhwof7sfi7db3lcdh5lrwrxluncbjxyksdnglx3pr3cmvugx6ep4",
  "is_wasm": true,
  "arguments": [
  {
    "value": "2",
    "arg_type": "Integer"
  },
  {
    "value": "5",
    "arg_type": "Integer"
  }
  ]
}'
```

Tank Verifier
```bash
curl -X POST "http://localhost:3000/api/prove" -H "Content-Type: application/json" -d '{
    "cid":"bafkreiho774aop3xjnaf22aomlaxrlxi652vbv2zfzqmjyai6kscg7fsei",
    "is_wasm":true,
    "arguments":[{"value":"10","arg_type":"Integer"},{"value":"10","arg_type":"Integer"},{"value":"300","arg_type":"Integer"},{"value":"30","arg_type":"Integer"},{"value":"5","arg_type":"Integer"},{"value":"5","arg_type":"Integer"},{"value":"1","arg_type":"Integer"},{"value":"1","arg_type":"Integer"},{"value":"15","arg_type":"Integer"},{"value":"15","arg_type":"Integer"},{"value":"1","arg_type":"Integer"},{"value":"1","arg_type":"Integer"}]
}'
```

**RUST Only**

Simple Factor
```
curl -X POST "http://localhost:3000/api/prove" -H "Content-Type: application/json" -d '{
  "cid": "bafkreicmml6a6n5yfw66njpdwa33kwjoghu7td4avf3sfyb2updshutee4",
  "is_wasm": false,
  "arguments": [
  {
    "value": "5",
    "arg_type": "Integer"
  },
  {
    "value": "5",
    "arg_type": "Integer"
  }
  ],
  "result_type": "Integer"
}'
```
