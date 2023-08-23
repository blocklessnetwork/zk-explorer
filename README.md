# risczero-zk-playground

Proof Generation API

`/api/prove`

```bash
curl -X POST "http://localhost:3000/api/prove" -H "Content-Type: application/json" -d '{
  "cid": "bafkreidhwof7sfi7db3lcdh5lrwrxluncbjxyksdnglx3pr3cmvugx6ep4",
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
